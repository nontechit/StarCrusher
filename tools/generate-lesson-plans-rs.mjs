// Generates src/lesson_plans_gen.rs from the curriculum corpus on D:.
//
// Reads K-5 Mathematics and English-Language-Arts lesson JSON from
// D:\lesson-plans and emits per-grade `&'static [LessonPlan]` tables that the
// game compiles in (WASM has no filesystem, so content must be static).
//
// Pre-Kindergarten stays hand-tuned in src/lesson_plans.rs — Frog Lane
// special-cases the PK-MATH-* lesson ids for its bespoke rendering.
//
// Vocabulary source order per lesson:
//   1. additional-materials\{Grade}\{Subject}\{ID}\vocabulary-builder.json
//      (rich: term/definition/partOfSpeech) when present — full coverage for
//      K, partial for 1st, absent for 2nd-5th as of 2026-06.
//   2. The lesson file's own `vocabulary` ({term, definition}) with a
//      part-of-speech heuristic.
//
// Usage: node tools/generate-lesson-plans-rs.mjs [corpusRoot]

import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

const CORPUS = process.argv[2] || "D:\\lesson-plans";
const OUT = path.join(path.dirname(fileURLToPath(import.meta.url)), "..", "src", "lesson_plans_gen.rs");

// Reading Snake constraints (see src/reading_snake.rs).
const MAX_WORD_LEN = 12;
const MIN_WORD_LEN = 3;
const MAX_DEF_LEN = 170;
const MAX_VOCAB_PER_LESSON = 6;

const GRADES = [
  { dir: "Kindergarten", rust: "Grade::Kindergarten", prefix: "K", mathGlob: /^K-MATH-\d+\.json$/i, elaGlob: /^K-ELA-\d+\.json$/i },
  { dir: "First", rust: "Grade::FirstGrade", prefix: "G1", mathGlob: /^1MATH-\d+\.json$/i, elaGlob: /^1ELA-\d+\.json$/i },
  { dir: "Second", rust: "Grade::SecondGrade", prefix: "G2", mathGlob: /^2MATH-\d+\.json$/i, elaGlob: /^2ELA-\d+\.json$/i },
  { dir: "Third", rust: "Grade::ThirdGrade", prefix: "G3", mathGlob: /^3MATH-\d+\.json$/i, elaGlob: /^3ELA-\d+\.json$/i },
  { dir: "Fourth", rust: "Grade::FourthGrade", prefix: "G4", mathGlob: /^4MATH-\d+\.json$/i, elaGlob: /^4ELA-\d+\.json$/i },
  { dir: "Fifth", rust: "Grade::FifthGrade", prefix: "G5", mathGlob: /^5MATH-\d+\.json$/i, elaGlob: /^5ELA-\d+\.json$/i },
];

// ── Helpers ──────────────────────────────────────────────────────────────────

// The corpus has three recurring damage classes: UTF-8 BOMs, truncated files
// (EOF before the root object closes), and one-too-many closing braces that
// end the root object mid-file. This string-aware pass repairs all three.
function repairJson(text) {
  const stack = [];
  let out = "";
  let inString = false;
  let escaped = false;
  for (let i = 0; i < text.length; i++) {
    const c = text[i];
    if (inString) {
      out += c;
      if (escaped) escaped = false;
      else if (c === "\\") escaped = true;
      else if (c === '"') inString = false;
      continue;
    }
    if (c === '"') {
      inString = true;
      out += c;
    } else if (c === "{" || c === "[") {
      stack.push(c);
      out += c;
    } else if (c === "}" || c === "]") {
      const open = c === "}" ? "{" : "[";
      if (stack.length === 0 || stack[stack.length - 1] !== open) continue; // drop stray closer
      // Drop a closer that would end the root while content still follows.
      if (stack.length === 1 && /\S/.test(text.slice(i + 1))) continue;
      stack.pop();
      out += c;
    } else {
      out += c;
    }
  }
  if (inString) out += '"';
  while (stack.length > 0) out += stack.pop() === "{" ? "}" : "]";
  return out;
}

function readJson(file) {
  const raw = fs.readFileSync(file, "utf8").replace(/^\uFEFF/, "");
  try {
    return JSON.parse(raw);
  } catch {
    try {
      const fixed = JSON.parse(repairJson(raw));
      console.warn(`  ~ repaired malformed JSON in ${path.basename(file)}`);
      return fixed;
    } catch (err) {
      console.warn(`  ! skipping unparseable ${file}: ${err.message}`);
      return null;
    }
  }
}

function listLessons(dir, glob) {
  if (!fs.existsSync(dir)) return [];
  return fs
    .readdirSync(dir)
    .filter((f) => glob.test(f) && !/-quiz\.json$/i.test(f))
    .sort((a, b) => a.localeCompare(b, undefined, { numeric: true }))
    .map((f) => path.join(dir, f));
}

function clean(s, maxLen = 0) {
  let out = String(s || "").replace(/\s+/g, " ").trim();
  if (maxLen > 0 && out.length > maxLen) {
    out = out.slice(0, maxLen - 1).trimEnd() + "\u2026";
  }
  return out;
}

function rustStr(s) {
  return `"${String(s).replace(/\\/g, "\\\\").replace(/"/g, '\\"')}"`;
}

// ── Math concept mapping (Frog Lane has 7 rendering modes) ──────────────────

function mathConcept(lesson) {
  const hay = `${lesson.title} ${(lesson.objectives || []).join(" ")}`.toLowerCase();
  if (/\bcolor/.test(hay)) return "Colors";
  if (/pattern|sequence|skip.?count/.test(hay)) return "Patterns";
  if (/sort|classif|categor|\bdata\b|graph|tally/.test(hay)) return "Sorting";
  if (/more|fewer|equal|greater|less than|compare numbers|estimat/.test(hay)) return "QuantComp";
  if (/measure|length|weight|height|size|longer|shorter|heavier|bigger|smaller/.test(hay)) return "SizeComp";
  if (/shape|geometr|polygon|angle|symmetr|area|perimeter|volume|triangle|quadrilateral/.test(hay)) return "Shapes";
  return "Counting"; // counting, place value, arithmetic drills, fractions...
}

// ── Part-of-speech heuristic for lesson-file vocab ───────────────────────────

const POS_EXACT = new Map(Object.entries({
  count: "verb", add: "verb", subtract: "verb", multiply: "verb", divide: "verb",
  compare: "verb", measure: "verb", estimate: "verb", round: "verb", predict: "verb",
  summarize: "verb", infer: "verb", revise: "verb", edit: "verb", sort: "verb",
  equal: "adjective", equivalent: "adjective", fluent: "adjective",
  more: "adjective", fewer: "adjective", greater: "adjective",
}));

function inferPos(term, definition) {
  const t = term.toLowerCase();
  if (POS_EXACT.has(t)) return POS_EXACT.get(t);
  const d = String(definition || "");
  if (/^to\s/i.test(d)) return "verb";
  if (/^(describing|having|able to|quick|careful|exact)\b/i.test(d)) return "adjective";
  if (/(tion|ment|ness|ity|er|or|ism)$/.test(t)) return "noun";
  if (/ly$/.test(t)) return "adverb";
  return "noun";
}

// ── Vocabulary assembly ──────────────────────────────────────────────────────

function builderVocab(gradeDir, subjectDir, lessonId) {
  const file = path.join(CORPUS, "additional-materials", gradeDir, subjectDir, lessonId, "vocabulary-builder.json");
  if (!fs.existsSync(file)) return null;
  const data = readJson(file);
  if (!data || !Array.isArray(data.vocabulary)) return null;
  return data.vocabulary.map((v) => ({
    term: clean(v.term),
    definition: clean(v.definition, MAX_DEF_LEN),
    pos: clean(v.partOfSpeech) || inferPos(v.term, v.definition),
  }));
}

function lessonVocab(lesson) {
  if (!Array.isArray(lesson.vocabulary)) return [];
  return lesson.vocabulary
    .filter((v) => v && v.term)
    .map((v) => ({
      term: clean(v.term),
      definition: clean(v.definition, MAX_DEF_LEN),
      pos: inferPos(v.term, v.definition),
    }));
}

/// Keep only terms Reading Snake can serve: one word, letters only, 3-12 chars.
function playable(vocab) {
  const seen = new Set();
  const out = [];
  for (const v of vocab) {
    const letters = v.term.replace(/[^A-Za-z]/g, "");
    if (letters !== v.term) continue; // multi-word / hyphenated terms drop out
    if (letters.length < MIN_WORD_LEN || letters.length > MAX_WORD_LEN) continue;
    const key = letters.toUpperCase();
    if (seen.has(key)) continue;
    seen.add(key);
    out.push(v);
    if (out.length >= MAX_VOCAB_PER_LESSON) break;
  }
  return out;
}

// ── Frog Lane strings per concept ────────────────────────────────────────────

const INSTRUCTION = {
  Counting: "Count each crossing! Say the number out loud.",
  Shapes: "Watch the shapes as you hop across!",
  Colors: "Name the colors as you cross each lane!",
  SizeComp: "Compare them: which is BIG, which is small?",
  QuantComp: "Which side has MORE? Which has FEWER?",
  Patterns: "Spot the repeating pattern in the lanes!",
  Sorting: "Group them! What do they have in common?",
};
const SUCCESS = {
  Counting: "Great counting! Lesson complete!",
  Shapes: "Great shape spotting! Lesson complete!",
  Colors: "You crossed all the colors! Lesson complete!",
  SizeComp: "Great comparing! Lesson complete!",
  QuantComp: "Great comparing! Lesson complete!",
  Patterns: "You spotted the patterns! Lesson complete!",
  Sorting: "Perfect sorting! Lesson complete!",
};

// ── Emission ─────────────────────────────────────────────────────────────────

function conceptLabel(lesson) {
  const ccss = lesson.standards && Array.isArray(lesson.standards.ccss) ? lesson.standards.ccss[0] : "";
  return ccss ? `CCSS ${clean(ccss, 28)}` : clean(lesson.subject || "Lesson", 28);
}

function emitVocab(vocab, indent) {
  if (vocab.length === 0) return `${indent}vocabulary: &[],\n`;
  let out = `${indent}vocabulary: &[\n`;
  for (const v of vocab) {
    out += `${indent}    VocabEntry { term: ${rustStr(v.term)}, part_of_speech: ${rustStr(v.pos)}, definition: ${rustStr(v.definition)} },\n`;
  }
  out += `${indent}],\n`;
  return out;
}

function emitLesson(lesson, gradeRust, subjectRust, vocab, math) {
  let out = "    LessonPlan {\n";
  out += `        id: ${rustStr(lesson.id)},\n`;
  out += `        title: ${rustStr(clean(lesson.title, 48))},\n`;
  out += `        grade: ${gradeRust},\n`;
  out += `        subject: ${subjectRust},\n`;
  out += `        concept: ${rustStr(conceptLabel(lesson))},\n`;
  if (math) {
    out += `        instruction: ${rustStr(INSTRUCTION[math.concept])},\n`;
    out += `        success: ${rustStr(SUCCESS[math.concept])},\n`;
  } else {
    out += `        instruction: ${rustStr("Spell the word from the lesson.")},\n`;
    out += `        success: ${rustStr("Great spelling!")},\n`;
  }
  out += emitVocab(vocab, "        ");
  if (math) {
    out += `        math: Some(MathLessonData { concept: MathConcept::${math.concept}, goal_hops: 5, start_count: ${math.startCount} }),\n`;
  } else {
    out += "        math: None,\n";
  }
  out += "    },\n";
  return out;
}

function generateGrade(g) {
  const mathFiles = listLessons(path.join(CORPUS, g.dir, "Mathematics"), g.mathGlob);
  const elaFiles = listLessons(path.join(CORPUS, g.dir, "English-Language-Arts"), g.elaGlob);

  let mathOut = "";
  let countingSeen = 0;
  let mathCount = 0;
  for (const file of mathFiles) {
    const lesson = readJson(file);
    if (!lesson || !lesson.id || !lesson.title) continue;
    const concept = mathConcept(lesson);
    // Counting-style lessons walk successive ranges, like the PK set does.
    const startCount = concept === "Counting" ? Math.min(countingSeen, 3) * 5 : 0;
    if (concept === "Counting") countingSeen++;
    const vocab = playable(lessonVocab(lesson));
    mathOut += emitLesson(lesson, g.rust, "Subject::Mathematics", vocab, { concept, startCount });
    mathCount++;
  }

  let litOut = "";
  let litCount = 0;
  for (const file of elaFiles) {
    const lesson = readJson(file);
    if (!lesson || !lesson.id || !lesson.title) continue;
    const rich = builderVocab(g.dir, "English-Language-Arts", lesson.id);
    const vocab = playable(rich || lessonVocab(lesson));
    if (vocab.length === 0) continue; // a literacy lesson with no playable words teaches nothing here
    litOut += emitLesson(lesson, g.rust, "Subject::Literacy", vocab, null);
    litCount++;
  }

  console.log(`${g.dir}: ${mathCount} math lessons, ${litCount} literacy lessons`);

  return (
    `pub const ${g.prefix}_MATH_LESSONS: &[LessonPlan] = &[\n${mathOut}];\n\n` +
    `pub const ${g.prefix}_LIT_LESSONS: &[LessonPlan] = &[\n${litOut}];\n\n`
  );
}

// ── Main ─────────────────────────────────────────────────────────────────────

let body = "";
for (const g of GRADES) {
  body += generateGrade(g);
}

const header = `// AUTO-GENERATED by tools/generate-lesson-plans-rs.mjs — DO NOT EDIT BY HAND.
//
// Per-grade lesson content for Kindergarten through 5th Grade, generated from
// the curriculum corpus at D:/lesson-plans (Mathematics + English-Language-Arts,
// with vocabulary enriched from additional-materials vocabulary-builder.json
// where available). Pre-Kindergarten content stays hand-tuned in
// lesson_plans.rs because Frog Lane special-cases the PK-MATH-* ids.
//
// Regenerate with: node tools/generate-lesson-plans-rs.mjs

use crate::lesson_plans::{LessonPlan, MathConcept, MathLessonData, Subject, VocabEntry};
use crate::levels::Grade;

`;

fs.writeFileSync(OUT, header + body, "utf8");
console.log(`wrote ${OUT}`);
