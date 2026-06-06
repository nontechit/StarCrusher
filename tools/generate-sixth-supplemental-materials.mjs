import fs from "node:fs";
import path from "node:path";

const gradeName = process.argv[2] || "Sixth";
const sourceRoot = `D:\\lesson-plans\\${gradeName}`;
const outputRoot = `D:\\lesson-plans\\additional-materials\\${gradeName}`;
const subjects = ["English-Language-Arts", "Mathematics", "Science", "Social-Studies"];

function readLesson(file) {
  const raw = fs.readFileSync(file, "utf8").trim();
  try {
    return JSON.parse(raw);
  } catch {
    const firstObject = extractFirstJsonObject(raw);
    if (firstObject) return JSON.parse(firstObject);
    let fixed = raw;
    const opens = (fixed.match(/{/g) || []).length;
    let closes = (fixed.match(/}/g) || []).length;
    while (closes < opens) {
      fixed += "}";
      closes += 1;
    }
    return JSON.parse(fixed);
  }
}

function extractFirstJsonObject(raw) {
  let depth = 0;
  let inString = false;
  let escaped = false;
  let started = false;
  for (let i = 0; i < raw.length; i += 1) {
    const ch = raw[i];
    if (!started) {
      if (ch === "{") {
        started = true;
        depth = 1;
      }
      continue;
    }
    if (escaped) {
      escaped = false;
      continue;
    }
    if (ch === "\\") {
      escaped = true;
      continue;
    }
    if (ch === "\"") {
      inString = !inString;
      continue;
    }
    if (inString) continue;
    if (ch === "{") depth += 1;
    if (ch === "}") depth -= 1;
    if (depth === 0) return raw.slice(raw.indexOf("{"), i + 1);
  }
  return null;
}

function md(text) {
  return String(text ?? "").replaceAll("|", "\\|");
}

function coreSkill(lesson) {
  return (lesson.objectives?.[0] || "Students will explain and apply the lesson concept").replace(/\.$/, "");
}

function concepts(lesson) {
  const list = [];
  for (const obj of lesson.objectives || []) list.push(String(obj).replace(/^Students will /, "").replace(/\.$/, ""));
  for (const step of lesson.procedure?.mainActivity?.steps || []) {
    if (list.length >= 5) break;
    const clean = String(step).replace(/^(Teach|Model|Introduce|Guided practice|Students|Small group|Real-world contexts):\s*/, "");
    if (clean.length > 12) list.push(clean.replace(/\.$/, ""));
  }
  return list.slice(0, 5);
}

function relatedTerms(subject) {
  const banks = {
    "English-Language-Arts": [
      ["inference", "A conclusion based on evidence and reasoning", "noun"],
      ["claim", "A statement that explains what you think or believe about a text", "noun"],
      ["context", "The words, events, or facts around an idea that help explain it", "noun"],
      ["interpretation", "An explanation of what something means", "noun"],
    ],
    Mathematics: [
      ["strategy", "A planned method for solving a problem", "noun"],
      ["justify", "To explain why an answer or method is correct", "verb"],
      ["model", "A diagram, table, equation, or example that represents a math idea", "noun"],
      ["reasonable", "Making sense based on the numbers and situation", "adjective"],
    ],
    Science: [
      ["evidence", "Information or observations that support an explanation", "noun"],
      ["system", "A group of parts that work together", "noun"],
      ["variable", "A factor that can change in an investigation", "noun"],
      ["data", "Facts, measurements, or observations collected during an investigation", "noun"],
    ],
    "Social-Studies": [
      ["cause", "An event or action that makes something happen", "noun"],
      ["effect", "A result or outcome of an event or action", "noun"],
      ["source", "A text, map, image, or object that gives information", "noun"],
      ["perspective", "A person's point of view about an event or issue", "noun"],
    ],
  };
  return (banks[subject] || banks["Social-Studies"]).map(([term, definition, partOfSpeech]) => ({ term, definition, partOfSpeech }));
}

function cleanLessonDetail(detail) {
  return String(detail || "")
    .replace(/^(Teach|Model|Introduce|Guided practice|Students|Small group|Real-world contexts|Ask|Discuss|Read|Present|Show|Pose|Set|Connect|Review|Quick check|Preview):\s*/i, "")
    .replace(/\s+/g, " ")
    .replace(/[.]+$/, "")
    .slice(0, 150);
}

function definitionCue(definition) {
  return String(definition || "the lesson idea")
    .replace(/\([^)]*\)/g, "")
    .replace(/;.*$/, "")
    .replace(/[.]+$/, "")
    .trim()
    .toLowerCase();
}

function vocabularyExample(term, definition, lesson, isRelated = false) {
  const detail = cleanLessonDetail(
    lesson.procedure?.mainActivity?.steps?.[1] ||
    lesson.procedure?.mainActivity?.steps?.[0] ||
    lesson.procedure?.warmup?.steps?.[0] ||
    lesson.title
  );
  const cue = definitionCue(definition);
  const lower = String(term).toLowerCase();

  const common = {
    evidence: `The class used measurements, observations, or source details as evidence to support a clear conclusion about ${lesson.title}.`,
    system: `A system has connected parts, so students looked at how each part worked together during ${lesson.title}.`,
    variable: `The variable changed in the investigation, so students watched how that change affected the result.`,
    data: `Students recorded data such as measurements, observations, or counts before making a conclusion.`,
    strategy: `One useful strategy was to choose a model or step-by-step method before solving the problem.`,
    justify: `Students justify an answer when they explain why their method or evidence proves it is correct.`,
    model: `A model such as a table, diagram, equation, or chart helped students represent the lesson idea.`,
    reasonable: `An answer is reasonable when it fits the numbers, evidence, or situation from the lesson.`,
    inference: `The reader made an inference by combining text evidence with what the character's actions suggested.`,
    claim: `The paragraph began with a claim that stated what the writer believed about the text.`,
    context: `The context around a word or event helped students understand its meaning in the passage.`,
    interpretation: `The student's interpretation explained what the author's choice suggested about the text.`,
    cause: `Students identified the cause by naming the event or action that made the change happen.`,
    effect: `Students described the effect by explaining what happened as a result of the event.`,
    source: `A source such as a map, text, image, or artifact gave students information to analyze.`,
    perspective: `A person's perspective shaped how they understood the event or issue.`,
  };
  if (common[lower]) return common[lower];

  if (lesson.subject === "Mathematics") {
    return `Students applied ${term} when they used ${detail} to show ${cue}.`;
  }
  if (lesson.subject === "Science") {
    return `During ${detail}, students used ${term} to describe or explain ${cue}.`;
  }
  if (lesson.subject === "Social-Studies") {
    return `When analyzing ${detail}, students used ${term} to explain ${cue}.`;
  }
  return `In a text discussion about ${detail}, students used ${term} to explain ${cue}.`;
}

function scenario(lesson) {
  switch (lesson.subject) {
    case "Mathematics":
      return `Use a table, equation, diagram, or clear arithmetic to solve a problem connected to ${lesson.title}.`;
    case "Science":
      return `Use evidence from an investigation or model to explain a pattern connected to ${lesson.title}.`;
    case "Social-Studies":
      return `Use a map, source, timeline, or example to explain a cause-and-effect relationship connected to ${lesson.title}.`;
    default:
      return `Use evidence from a text to explain an idea connected to ${lesson.title}.`;
  }
}

function applicationPrompt(lesson, activity) {
  switch (lesson.subject) {
    case "Mathematics":
      return `Apply the method or representation from this lesson activity to solve a related problem: ${activity}`;
    case "Science":
      return `Use the scientific idea from this lesson activity to explain a related observation or pattern: ${activity}`;
    case "Social-Studies":
      return `Use the historical, geographic, civic, or economic idea from this lesson activity to explain a related situation: ${activity}`;
    default:
      return `Use the reading or writing skill from this lesson activity to analyze a related text example: ${activity}`;
  }
}

function exercise(number, objective, type, question, answer, explanation, difficulty, options) {
  const item = { number, objective, type, question, answer, explanation, difficulty };
  if (options) item.options = options;
  return item;
}

function practiceExercises(lesson) {
  const objectives = lesson.objectives || [];
  const vocab = lesson.vocabulary || [];
  const term = vocab[0]?.term || "key concept";
  const definition = vocab[0]?.definition || "an important idea from the lesson";
  const items = [
    exercise(1, objectives[0] || coreSkill(lesson), "multiple-choice", `Which statement best matches the meaning of '${term}' in this lesson?`, `A) ${definition}`, `The correct answer uses the lesson definition of '${term}'; the other choices are related words or incomplete ideas.`, "easy", [`A) ${definition}`, "B) A personal opinion with no evidence", "C) A random detail that does not affect the lesson", "D) A question that cannot be answered"]),
    exercise(2, objectives[0] || coreSkill(lesson), "short-answer", `Explain why ${lesson.title} is important in one or two complete sentences.`, "A strong answer names the main skill or concept and explains how it helps students understand or solve lesson tasks.", "The response should connect directly to the lesson title and objective instead of giving a general classroom answer.", "medium"),
  ];
  if (objectives[1]) items.push(exercise(3, objectives[1], "fill-in-blank", "Fill in the blank: To meet this objective, a student should use lesson evidence and accurate vocabulary to __________ their thinking.", "explain", "Students need to explain, not just list, their thinking so the reasoning is clear.", "easy"));
  if (objectives[2]) items.push(exercise(4, objectives[2], "true-false", "True or False: A complete response should include evidence or reasoning that supports the answer.", "True", "The lesson expects students to support answers with evidence, examples, calculations, or reasoning.", "easy"));
  items.push(exercise(5, coreSkill(lesson), "problem-solving", scenario(lesson), "A complete answer applies the lesson concept, shows evidence or steps, and explains why the result makes sense.", "This checks whether students can transfer the lesson skill to a realistic task.", "hard"));
  items.push(exercise(6, coreSkill(lesson), "short-answer", "Describe one common mistake a student might make during this lesson and how to fix it.", "A strong answer identifies a specific misconception and gives the correct method or understanding.", "This reinforces accuracy and helps students monitor their own work.", "medium"));
  return items;
}

function writeContentBrief(lesson, outDir) {
  const defs = (lesson.vocabulary || []).map((v) => `| ${md(v.term)} | ${md(v.definition)} |`);
  const prior = lesson.prerequisites?.join("; ") || "Foundational skills from earlier lessons";
  const next = lesson.extensions?.join("; ") || "More independent application of this skill";
  const lines = [
    `# Content Brief: ${lesson.title}`,
    `**Lesson ID:** ${lesson.id} | **Grade:** ${lesson.grade || gradeName} | **Subject:** ${lesson.subject} | **Duration:** ${lesson.duration}`,
    "",
    "## Key Concepts",
    ...concepts(lesson).map((c) => `- ${c}`),
    "",
    "## Important Definitions",
    "| Term | Definition |",
    "|------|-----------|",
    ...(defs.length ? defs : ["| key concept | An important idea from the lesson |"]),
    "",
    "## Worked Examples",
    `${scenario(lesson)} A teacher model should name the goal, show the steps or evidence, and end with a sentence explaining why the answer fits the lesson objective.`,
    "",
    "## Common Misconceptions",
    "- **Misconception:** Students may give an answer without showing evidence, steps, or reasoning.",
    "  **Correction:** Require students to connect each answer to a quote, example, calculation, observation, map feature, or source detail from the lesson.",
    "",
    "## Lesson Connections",
    `- **Prior lesson:** ${prior}`,
    `- **Next lesson:** ${next}`,
  ];
  fs.writeFileSync(path.join(outDir, "content-brief.md"), lines.join("\r\n"));
}

function writeVocabulary(lesson, outDir) {
  const seen = new Set();
  const vocabulary = [];
  for (const v of lesson.vocabulary || []) {
    if (seen.has(v.term)) continue;
    seen.add(v.term);
      vocabulary.push({
        term: v.term,
        definition: v.definition,
        partOfSpeech: "noun",
        exampleSentence: vocabularyExample(v.term, v.definition, lesson),
        activityType: "define-from-context",
      });
  }
  for (const r of relatedTerms(lesson.subject)) {
    if (seen.has(r.term)) continue;
    seen.add(r.term);
      vocabulary.push({
        term: r.term,
        definition: r.definition,
        partOfSpeech: r.partOfSpeech,
        exampleSentence: vocabularyExample(r.term, r.definition, lesson, true),
        activityType: "use-in-sentence",
      });
  }
  fs.writeFileSync(path.join(outDir, "vocabulary-builder.json"), JSON.stringify({ lessonId: lesson.id, vocabulary }, null, 2));
}

function writePractice(lesson, outDir) {
  fs.writeFileSync(path.join(outDir, "practice-exercises.json"), JSON.stringify({
    lessonId: lesson.id,
    grade: lesson.grade || gradeName,
    subject: lesson.subject,
    exercises: practiceExercises(lesson),
  }, null, 2));
}

function writeStudyGuide(lesson, outDir) {
  const vocab = (lesson.vocabulary || []).slice(0, 4);
  const lines = [`# Study Guide: ${lesson.title}`, "**Name:** _______________ **Date:** _______________", ""];
  (lesson.objectives || [coreSkill(lesson)]).forEach((objective, index) => {
    lines.push(`## Objective ${index + 1}: ${objective}`, "", "### Key Terms");
    for (const v of vocab) lines.push(`- **${v.term}:** ${v.definition}`);
    lines.push("", "### What You Need to Know", "This objective asks you to use the lesson ideas carefully, show your thinking, and support your answer with the right evidence, steps, or examples.", "", "### Worked Example", "Start by identifying what the question is asking. Next, choose a lesson term, fact, quote, calculation, model, or source detail that helps. Then explain how that evidence supports your answer.", "", "---", "");
  });
  lines.push("## Try It! Practice Problems", "", "1. Write one sentence that explains the main idea of this lesson.", "2. Choose one vocabulary word and use it correctly in a sentence.", "3. Solve or explain one example from class using clear steps.", "4. Describe one mistake to avoid when completing this lesson skill.", "", "---", "", "## Self-Check Questions", "", "1. What evidence or steps support your answer? -> *Answer: Use a specific quote, example, calculation, observation, map detail, or source detail.*", "2. How does one vocabulary word connect to the lesson objective? -> *Answer: Define the word and explain how it helps with the skill.*", "3. Why is explaining your reasoning important? -> *Answer: It shows how you know your answer is accurate.*", "", "---", "", "## Answer Key (Try It!)", "1. Answers should name the lesson topic and a specific skill.", "2. Sentences should use the word accurately and clearly.", "3. Answers should show steps, evidence, or reasoning.", "4. Answers should name a real mistake and explain the correction.");
  fs.writeFileSync(path.join(outDir, "study-guide.md"), lines.join("\r\n"));
}

function writeExitTicket(lesson, outDir) {
  const term = lesson.vocabulary?.[0]?.term || "lesson concept";
  const def = lesson.vocabulary?.[0]?.definition || "an important lesson idea";
  const questions = [
    { number: 1, type: "multiple-choice", question: `Which answer best defines '${term}' for this lesson?`, options: [`A) ${def}`, "B) A detail with no connection to the lesson", "C) A personal preference", "D) A question with no evidence"], answer: `A) ${def}` },
    { number: 2, type: "short-answer", question: "Use one lesson vocabulary word to explain part of today's objective.", answer: "A complete answer uses the vocabulary word correctly and connects it to the lesson objective." },
    { number: 3, type: "true-false", question: "A strong answer should include evidence, steps, or reasoning from the lesson.", answer: "True - support shows why the answer is correct." },
  ];
  fs.writeFileSync(path.join(outDir, "exit-ticket.json"), JSON.stringify({ lessonId: lesson.id, title: `Exit Ticket: ${lesson.title}`, questions }, null, 2));
}

function writeQuiz(lesson, outDir) {
  const objectives = lesson.objectives || [coreSkill(lesson)];
  const vocab = lesson.vocabulary || [];
  const term1 = vocab[0]?.term || "key concept";
  const def1 = vocab[0]?.definition || "an important idea from the lesson";
  const term2 = vocab[1]?.term || "evidence";
  const def2 = vocab[1]?.definition || "support for an answer";
  const term3 = vocab[2]?.term || term1;
  const def3 = vocab[2]?.definition || def1;
  const term4 = vocab[3]?.term || term2;
  const def4 = vocab[3]?.definition || def2;
  const warmup = lesson.procedure?.warmup?.steps?.[0] || lesson.procedure?.warmup?.steps?.[1] || `the opening activity for ${lesson.title}`;
  const main1 = lesson.procedure?.mainActivity?.steps?.[0] || `the main activity for ${lesson.title}`;
  const main2 = lesson.procedure?.mainActivity?.steps?.[1] || main1;
  const main3 = lesson.procedure?.mainActivity?.steps?.[2] || main2;
  const wrap = lesson.procedure?.wrapUp?.steps?.[0] || `the closing discussion for ${lesson.title}`;
  const material = lesson.materials?.[0] || "lesson materials";
  const assessment = lesson.assessment?.summative?.[0] || objectives[0];
  const prior = lesson.prerequisites?.[0] || "prior learning";
  const extension = lesson.extensions?.[0] || "an extension activity";
  const distractorDefs = [
    "A personal preference that does not need support",
    "A classroom routine unrelated to the lesson content",
    "A guess made before reading or solving",
    "A copied sentence with no explanation",
  ];
  const questions = [
    { number: 1, type: "multiple-choice", question: `In ${lesson.title}, what does '${term1}' mean?`, options: [`A) ${def1}`, `B) ${distractorDefs[0]}`, `C) ${distractorDefs[1]}`, `D) ${distractorDefs[2]}`], answer: `A) ${def1}`, points: 2 },
    { number: 2, type: "multiple-choice", question: `Which definition best matches '${term2}' as used in this lesson?`, options: [`A) ${distractorDefs[3]}`, `B) ${def2}`, `C) ${distractorDefs[2]}`, `D) ${distractorDefs[1]}`], answer: `B) ${def2}`, points: 2 },
    { number: 3, type: "multiple-choice", question: `The lesson uses '${material}' mainly to support which learning goal?`, options: [`A) ${objectives[0] || assessment}`, `B) Memorizing unrelated facts`, `C) Avoiding the lesson vocabulary`, `D) Replacing the main activity`], answer: `A) ${objectives[0] || assessment}`, points: 2 },
    { number: 4, type: "multiple-choice", question: `Which activity detail comes directly from the lesson plan for ${lesson.id}?`, options: [`A) ${main1}`, `B) Students skip the main activity and only take notes`, `C) Students study a topic from a different grade`, `D) Students complete an unrelated art project`], answer: `A) ${main1}`, points: 2 },
    { number: 5, type: "true-false", question: `True or False: The lesson connects prior learning such as '${prior}' to new work on ${lesson.title}.`, answer: "True", points: 1 },
    { number: 6, type: "fill-in-blank", question: `Fill in the blank: One important lesson term is '${term3}', which means __________.`, answer: def3, points: 1 },
    { number: 7, type: "short-answer", question: `Explain how this activity supports the lesson objective '${objectives[0]}': ${main2}`, answer: `A complete response explains that the activity helps students practice '${objectives[0]}' and uses details from the activity, such as ${main2}.`, points: 3, rubric: `Full credit (3 pts): connects the activity to '${objectives[0]}', includes a specific detail from the lesson activity, and explains the learning purpose. Partial credit (1-2 pts): names the activity or objective but gives a thin explanation. No credit: off-topic or missing.` },
    { number: 8, type: "short-answer", question: `Use the vocabulary term '${term4}' in a sentence that fits ${lesson.title}.`, answer: `A correct sentence uses '${term4}' to mean '${def4}' and connects it to the lesson topic.`, points: 2, rubric: `Full credit (2 pts): uses '${term4}' accurately and connects it to ${lesson.title}. Partial credit (1 pt): sentence is partly correct but vague. No credit: inaccurate or missing.` },
    { number: 9, type: "problem-solving", question: applicationPrompt(lesson, main3), answer: `A complete response applies the lesson concept to the task, uses a detail from '${main3}', and explains why the answer fits ${lesson.title}.`, points: 4, rubric: `Full credit (4 pts): applies the correct lesson concept, uses a specific detail from '${main3}', and explains the reasoning. Partial credit (1-3 pts): uses the right topic but has incomplete evidence, steps, or explanation. No credit: off-topic or unsupported.` },
    { number: 10, type: "short-answer", question: `The wrap-up includes: ${wrap} Explain what a teacher could learn from this response about whether students met the assessment target '${assessment}'.`, answer: `A strong answer explains how the wrap-up evidence can show progress toward '${assessment}' and names what the teacher should look for in student responses.`, points: 1, rubric: `Full credit (1 pt): connects the wrap-up to '${assessment}' and identifies evidence of understanding. No credit: off-topic or missing.` },
  ];
  fs.writeFileSync(path.join(outDir, `${lesson.id}-quiz-replacement.json`), JSON.stringify({
    id: `${lesson.id}-quiz`,
    lessonId: lesson.id,
    title: `${lesson.title} - Quiz`,
    grade: lesson.grade || gradeName,
    subject: lesson.subject,
    type: "summative-quiz",
    totalPoints: 20,
    timeAllowed: lesson.duration,
    questions,
  }, null, 2));
}

let count = 0;
for (const subject of subjects) {
  const folder = path.join(sourceRoot, subject);
  const files = fs.readdirSync(folder).filter((name) => name.endsWith(".json") && !name.endsWith("-quiz.json")).sort();
  for (const file of files) {
    const lesson = readLesson(path.join(folder, file));
    const outDir = path.join(outputRoot, subject, lesson.id);
    fs.mkdirSync(outDir, { recursive: true });
    writeContentBrief(lesson, outDir);
    writePractice(lesson, outDir);
    writeVocabulary(lesson, outDir);
    writeStudyGuide(lesson, outDir);
    writeExitTicket(lesson, outDir);
    writeQuiz(lesson, outDir);
    count += 6;
  }
}

console.log(`Generated ${count} supplemental files in ${outputRoot}`);
