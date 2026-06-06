import fs from "node:fs";
import path from "node:path";

const grades = ["Freshman", "Sophomore", "Junior", "Senior"];

function readJson(file) {
  const raw = fs.readFileSync(file, "utf8").replace(/^\uFEFF/, "").trim();
  try {
    return JSON.parse(raw);
  } catch {
    const start = raw.indexOf("{");
    let depth = 0;
    let inString = false;
    let escaped = false;
    for (let i = start; i < raw.length; i += 1) {
      const ch = raw[i];
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
      if (depth === 0) return JSON.parse(raw.slice(start, i + 1));
    }
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

function parseLooseLesson(file) {
  const raw = fs.readFileSync(file, "utf8").replace(/^\uFEFF/, "");
  const prop = (name) => {
    const match = raw.match(new RegExp(`"${name}"\\s*:\\s*"([\\s\\S]*?)"`));
    return match ? match[1].replace(/\s+/g, " ").trim() : "";
  };
  const arrayProp = (name) => {
    const match = raw.match(new RegExp(`"${name}"\\s*:\\s*\\[([\\s\\S]*?)\\]`));
    if (!match) return [];
    return [...match[1].matchAll(/"([^"]+)"/g)].map((m) => m[1].replace(/\s+/g, " ").trim());
  };
  const sectionSteps = (section) => {
    const sectionMatch = raw.match(new RegExp(`"${section}"\\s*:\\s*\\{([\\s\\S]*?)(?:"mainActivity"|"wrapUp"|"assessment"|"differentiation"|$)`));
    if (!sectionMatch) return [];
    const stepsMatch = sectionMatch[1].match(/"steps"\s*:\s*\[([\s\S]*?)\]/);
    if (!stepsMatch) return [];
    return [...stepsMatch[1].matchAll(/"([^"]+)"/g)].map((m) => m[1].replace(/\s+/g, " ").trim());
  };
  const vocab = [];
  for (const match of raw.matchAll(/"term"\s*:\s*"([^"]+)"[\s\S]{0,120}?(?:"definition"|Definition")\s*:\s*"([^"]+)"/g)) {
    vocab.push({ term: match[1].trim(), definition: match[2].trim() });
  }
  const parts = file.split(/[\\/]/);
  const grade = prop("grade") || parts.find((p) => grades.includes(p)) || "";
  const subject = prop("subject") || path.basename(path.dirname(file));
  return {
    id: prop("id") || path.basename(file, ".json"),
    title: prop("title") || `${path.basename(file, ".json")} Lesson`,
    grade,
    subject,
    duration: prop("duration") || "50 minutes",
    prerequisites: arrayProp("prerequisites"),
    objectives: arrayProp("objectives"),
    standards: { ccss: arrayProp("ccss"), state: arrayProp("state") },
    vocabulary: vocab,
    materials: arrayProp("materials"),
    procedure: {
      warmup: { steps: sectionSteps("warmup") },
      mainActivity: { steps: sectionSteps("mainActivity") },
      wrapUp: { steps: sectionSteps("wrapUp") },
    },
    assessment: {
      formative: arrayProp("formative"),
      summative: arrayProp("summative"),
    },
    extensions: arrayProp("extensions"),
    homework: prop("homework"),
  };
}

function readLesson(file) {
  try {
    return readJson(file);
  } catch {
    return parseLooseLesson(file);
  }
}

function md(text) {
  return String(text ?? "").replaceAll("|", "\\|");
}

function clean(text) {
  return String(text || "")
    .replace(/^(Teach|Model|Introduce|Guided practice|Students|Small group|Real-world contexts|Ask|Discuss|Read|Present|Show|Pose|Set|Connect|Review|Quick check|Preview|Teacher model|Independent practice):\s*/i, "")
    .replace(/\s+/g, " ")
    .replace(/[.]+$/, "")
    .trim();
}

function short(text, max = 190) {
  const value = clean(text);
  return value.length > max ? `${value.slice(0, max - 3)}...` : value;
}

function concepts(lesson) {
  const list = [];
  for (const objective of lesson.objectives || []) list.push(String(objective).replace(/^Students will /, "").replace(/[.]+$/, ""));
  for (const step of lesson.procedure?.mainActivity?.steps || []) {
    if (list.length >= 5) break;
    list.push(short(step));
  }
  return list.slice(0, 5);
}

function relatedTerms(lesson) {
  const subject = String(lesson.subject || "");
  if (subject.includes("Math")) {
    return [
      ["reasoning", "A clear explanation of why a mathematical method or answer makes sense", "noun"],
      ["representation", "A graph, table, equation, diagram, or model that shows a mathematical idea", "noun"],
      ["constraint", "A condition or limit that affects a mathematical situation", "noun"],
      ["precision", "Careful and accurate use of numbers, units, symbols, and language", "noun"],
    ];
  }
  if (subject.includes("Science")) {
    return [
      ["evidence", "Data or observations used to support a scientific explanation", "noun"],
      ["model", "A simplified representation used to explain or predict a system", "noun"],
      ["variable", "A factor that can change in an investigation", "noun"],
      ["claim", "A scientific statement supported by evidence and reasoning", "noun"],
    ];
  }
  if (subject.includes("Social")) {
    return [
      ["context", "The conditions and events surrounding a historical issue or source", "noun"],
      ["causation", "The relationship between events in which one contributes to another", "noun"],
      ["perspective", "A point of view shaped by a person's experiences, values, and position", "noun"],
      ["source", "A document, image, map, artifact, or data set used as evidence", "noun"],
    ];
  }
  if (subject.includes("World") || subject.includes("Language")) {
    return [
      ["register", "The level of formality used in communication", "noun"],
      ["comprehension", "Understanding spoken, written, or signed language", "noun"],
      ["fluency", "The ability to communicate smoothly and accurately", "noun"],
      ["culture", "The practices, values, and products shared by a community", "noun"],
    ];
  }
  return [
    ["analysis", "A careful explanation of how parts work together to create meaning or effect", "noun"],
    ["evidence", "Specific details used to support an idea or interpretation", "noun"],
    ["audience", "The people a text, speech, design, or project is created for", "noun"],
    ["purpose", "The reason a creator writes, speaks, designs, or performs", "noun"],
  ];
}

function exampleSentence(term, definition, lesson) {
  const lower = term.toLowerCase();
  const exact = {
    evidence: "The evidence included a quoted passage, data point, or observation that supported the claim.",
    model: "The model showed the important parts of the system without including every small detail.",
    variable: "The variable changed during the investigation, so students measured how it affected the result.",
    claim: "The claim stated the main idea that the rest of the response needed to prove.",
    analysis: "The analysis explained how the author's choices shaped the reader's understanding.",
    audience: "The audience for the presentation shaped the speaker's examples and tone.",
    purpose: "The purpose of the speech was to persuade classmates to support the proposal.",
    context: "The context includes the events, conditions, and ideas that surrounded the source or issue.",
    causation: "Causation links one event to another by explaining how the first contributed to the second.",
    perspective: "Perspective shaped how each source described the same event.",
    source: "The source provided evidence about the event from a specific point of view.",
    reasoning: "The reasoning explained why each algebra step kept the equation balanced.",
    representation: "The graph was a representation of the same relationship shown in the equation.",
    constraint: "The budget was a constraint because it limited which solution was possible.",
    precision: "Precision matters when a student labels units and uses exact vocabulary.",
    register: "The speaker used a formal register when addressing the teacher.",
    comprehension: "Comprehension improved when students listened for familiar words and context clues.",
    fluency: "Fluency grew as students practiced the conversation several times.",
    culture: "Culture influenced the greeting, gesture, or custom used in the conversation.",
  };
  if (exact[lower]) return exact[lower];
  const subject = String(lesson.subject || "");
  if (subject.includes("Math")) return `Students used ${term} to solve a problem involving ${lesson.title}.`;
  if (subject.includes("Science")) return `The lab discussion used ${term} to explain ${definition.toLowerCase().replace(/[.]+$/, "")}.`;
  if (subject.includes("Social")) return `${term} shaped the historical or civic issue examined in ${lesson.title}.`;
  if (subject.includes("World") || subject.includes("Language")) return `Students practiced ${term} during a communication task in ${lesson.title}.`;
  return `The response used ${term} to analyze a key idea in ${lesson.title}.`;
}

function applicationPrompt(lesson, activity) {
  const subject = String(lesson.subject || "");
  if (subject.includes("Math")) return `Apply the method from this activity to solve a related problem: ${activity}`;
  if (subject.includes("Science")) return `Use the scientific idea from this activity to explain a related observation or data pattern: ${activity}`;
  if (subject.includes("Social")) return `Use the historical, civic, geographic, or economic idea from this activity to explain a related situation: ${activity}`;
  if (subject.includes("World") || subject.includes("Language")) return `Use the language skill from this activity in a related communication task: ${activity}`;
  if (subject.includes("Elective")) return `Apply the skill from this activity to a realistic scenario: ${activity}`;
  return `Use the reading, writing, speaking, or analysis skill from this activity to respond to a related text or task: ${activity}`;
}

function writeContentBrief(lesson, outDir) {
  const defs = (lesson.vocabulary || []).map((v) => `| ${md(v.term)} | ${md(v.definition)} |`);
  const prior = lesson.prerequisites?.join("; ") || "Relevant prior course knowledge";
  const next = lesson.extensions?.join("; ") || lesson.homework || "More independent application of the skill";
  const lines = [
    `# Content Brief: ${lesson.title}`,
    `**Lesson ID:** ${lesson.id} | **Grade:** ${lesson.grade} | **Subject:** ${lesson.subject} | **Duration:** ${lesson.duration}`,
    "",
    "## Key Concepts",
    ...concepts(lesson).map((item) => `- ${item}`),
    "",
    "## Important Definitions",
    "| Term | Definition |",
    "|------|------------|",
    ...(defs.length ? defs : ["| key concept | A central idea or skill from the lesson |"]),
    "",
    "## Worked Examples",
    `${applicationPrompt(lesson, short(lesson.procedure?.mainActivity?.steps?.[0] || lesson.title))} A strong model names the task, applies the relevant concept or process, and explains the reasoning with lesson-specific evidence.`,
    "",
    "## Common Misconceptions",
    "- **Misconception:** Students may describe the topic generally without using the lesson's specific vocabulary, evidence, process, or criteria.",
    "  **Correction:** Require answers to reference a lesson term, activity detail, source, data point, text feature, procedure, or model.",
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
  for (const item of lesson.vocabulary || []) {
    if (!item.term || seen.has(item.term)) continue;
    seen.add(item.term);
    vocabulary.push({
      term: item.term,
      definition: item.definition,
      partOfSpeech: "noun",
      exampleSentence: exampleSentence(item.term, item.definition || "", lesson),
      activityType: "define-from-context",
    });
  }
  for (const [term, definition, partOfSpeech] of relatedTerms(lesson)) {
    if (seen.has(term)) continue;
    seen.add(term);
    vocabulary.push({
      term,
      definition,
      partOfSpeech,
      exampleSentence: exampleSentence(term, definition, lesson),
      activityType: "use-in-sentence",
    });
  }
  fs.writeFileSync(path.join(outDir, "vocabulary-builder.json"), JSON.stringify({ lessonId: lesson.id, vocabulary }, null, 2));
}

function practiceExercises(lesson) {
  const vocab = lesson.vocabulary || [];
  const objectives = lesson.objectives || [`Students will explain ${lesson.title}`];
  const term = vocab[0]?.term || "key concept";
  const definition = vocab[0]?.definition || "a central idea from the lesson";
  const activity = short(lesson.procedure?.mainActivity?.steps?.[0] || lesson.title);
  return [
    {
      number: 1,
      objective: objectives[0],
      type: "multiple-choice",
      question: `Which answer best defines '${term}' in ${lesson.title}?`,
      options: [`A) ${definition}`, "B) A topic from another course", "C) A personal preference", "D) A future unit not addressed here"],
      answer: `A) ${definition}`,
      explanation: `The correct answer matches the lesson definition of '${term}'.`,
      difficulty: "easy",
    },
    {
      number: 2,
      objective: objectives[0],
      type: "short-answer",
      question: `Explain how this activity supports the lesson goal: ${activity}`,
      answer: "A complete answer connects the activity to the lesson objective and uses a specific lesson detail.",
      explanation: "This checks whether students understand why the activity matters.",
      difficulty: "medium",
    },
    {
      number: 3,
      objective: objectives[1] || objectives[0],
      type: "fill-in-blank",
      question: `Fill in the blank: A strong response about ${lesson.title} should use accurate vocabulary and specific __________ from the lesson.`,
      answer: "evidence",
      explanation: "High school responses need evidence, data, examples, or reasoning.",
      difficulty: "easy",
    },
    {
      number: 4,
      objective: objectives[1] || objectives[0],
      type: "true-false",
      question: `True or False: A response can fully meet the objectives of ${lesson.title} without referring to any lesson-specific content.`,
      answer: "False",
      explanation: "The response must use lesson-specific vocabulary, methods, evidence, or examples.",
      difficulty: "easy",
    },
    {
      number: 5,
      objective: objectives[2] || objectives[0],
      type: "problem-solving",
      question: applicationPrompt(lesson, short(lesson.procedure?.mainActivity?.steps?.[1] || activity)),
      answer: "A complete answer applies the lesson skill accurately and explains the reasoning.",
      explanation: "This requires transfer of the lesson skill to a related task.",
      difficulty: "hard",
    },
    {
      number: 6,
      objective: objectives[0],
      type: "short-answer",
      question: "Name one likely misconception from this lesson and explain the correction.",
      answer: "A strong response names a lesson-specific misconception and gives the accurate correction.",
      explanation: "This checks conceptual precision.",
      difficulty: "medium",
    },
  ];
}

function writePractice(lesson, outDir) {
  fs.writeFileSync(path.join(outDir, "practice-exercises.json"), JSON.stringify({
    lessonId: lesson.id,
    grade: lesson.grade,
    subject: lesson.subject,
    exercises: practiceExercises(lesson),
  }, null, 2));
}

function writeStudyGuide(lesson, outDir) {
  const vocab = (lesson.vocabulary || []).slice(0, 5);
  const lines = [`# Study Guide: ${lesson.title}`, "**Name:** _______________ **Date:** _______________", ""];
  (lesson.objectives || [`Students will explain ${lesson.title}`]).forEach((objective, index) => {
    lines.push(`## Objective ${index + 1}: ${objective}`, "", "### Key Terms");
    for (const item of vocab) lines.push(`- **${item.term}:** ${item.definition}`);
    lines.push("", "### What You Need to Know", "Focus on the specific vocabulary, evidence, procedure, text, model, or criteria from the lesson. A strong answer explains both what you know and how you know it.", "", "### Worked Example", applicationPrompt(lesson, short(lesson.procedure?.mainActivity?.steps?.[index] || lesson.procedure?.mainActivity?.steps?.[0] || lesson.title)), "", "---", "");
  });
  lines.push("## Try It! Practice Problems", "", "1. Define one key vocabulary term in your own words.", "2. Explain how one lesson activity supports an objective.", "3. Apply the lesson skill to a related example.", "4. Identify one misconception and correct it.", "", "---", "", "## Self-Check Questions", "", "1. Did I use lesson-specific vocabulary? -> *Answer: Yes, if the response names and correctly uses a lesson term.*", "2. Did I support my thinking? -> *Answer: Yes, if the response includes evidence, data, examples, steps, or reasoning.*", "3. Did I connect the activity to the objective? -> *Answer: Yes, if the response explains why the activity builds the target skill.*", "", "---", "", "## Answer Key (Try It!)", "1. Definitions should be accurate and specific.", "2. Answers should connect the activity to an objective.", "3. Answers should apply the skill with clear reasoning.", "4. Answers should name a real misconception and provide the correction.");
  fs.writeFileSync(path.join(outDir, "study-guide.md"), lines.join("\r\n"));
}

function writeExitTicket(lesson, outDir) {
  const vocab = lesson.vocabulary || [];
  const term = vocab[0]?.term || "key concept";
  const definition = vocab[0]?.definition || "a central idea from the lesson";
  const questions = [
    { number: 1, type: "multiple-choice", question: `Which answer best defines '${term}' in ${lesson.title}?`, options: [`A) ${definition}`, "B) An unrelated review topic", "C) A personal opinion", "D) A future unit topic"], answer: `A) ${definition}` },
    { number: 2, type: "short-answer", question: `Use one detail from ${lesson.title} to explain what you learned today.`, answer: "A complete answer uses a lesson-specific term, activity detail, evidence, data point, or method." },
    { number: 3, type: "true-false", question: `A strong response to this lesson should connect the objective to specific evidence, reasoning, or process from class.`, answer: "True - strong responses are specific and supported." },
  ];
  fs.writeFileSync(path.join(outDir, "exit-ticket.json"), JSON.stringify({ lessonId: lesson.id, title: `Exit Ticket: ${lesson.title}`, questions }, null, 2));
}

function writeQuiz(lesson, outDir) {
  const vocab = lesson.vocabulary || [];
  const term1 = vocab[0]?.term || "key concept";
  const def1 = vocab[0]?.definition || "a central idea from the lesson";
  const term2 = vocab[1]?.term || term1;
  const def2 = vocab[1]?.definition || def1;
  const objectives = lesson.objectives || [`Students will explain ${lesson.title}`];
  const activity1 = short(lesson.procedure?.mainActivity?.steps?.[0] || lesson.title);
  const activity2 = short(lesson.procedure?.mainActivity?.steps?.[1] || activity1);
  const assessment = short(lesson.assessment?.summative?.[0] || objectives[0]);
  const questions = [
    { number: 1, type: "multiple-choice", question: `In ${lesson.title}, what does '${term1}' mean?`, options: [`A) ${def1}`, "B) An unrelated review topic", "C) A personal preference", "D) A future unit topic"], answer: `A) ${def1}`, points: 2 },
    { number: 2, type: "multiple-choice", question: `Which activity detail comes from ${lesson.id}?`, options: [`A) ${activity1}`, "B) Students complete unrelated memorization only", "C) Students skip the lesson skill", "D) Students work on a different course topic"], answer: `A) ${activity1}`, points: 2 },
    { number: 3, type: "short-answer", question: `Explain how '${activity2}' supports this objective: '${objectives[0]}'.`, answer: "A complete response connects the activity to the objective and includes a specific lesson detail.", points: 3, rubric: "Full credit (3 pts): clear objective connection, specific lesson detail, and accurate explanation. Partial credit (1-2 pts): incomplete connection or vague detail. No credit: off-topic or missing." },
    { number: 4, type: "short-answer", question: `Define '${term2}' and use it correctly in a sentence connected to ${lesson.title}.`, answer: `A correct response defines '${term2}' as '${def2}' and uses it accurately in context.`, points: 2, rubric: "Full credit (2 pts): accurate definition and sentence. Partial credit (1 pt): partly accurate. No credit: inaccurate or missing." },
    { number: 5, type: "problem-solving", question: applicationPrompt(lesson, short(lesson.procedure?.mainActivity?.steps?.[2] || activity2)), answer: "A complete response applies the lesson skill accurately and explains the reasoning.", points: 4, rubric: "Full credit (4 pts): accurate application, lesson-specific detail, and clear reasoning. Partial credit (1-3 pts): partly correct but incomplete. No credit: off-topic or unsupported." },
    { number: 6, type: "true-false", question: `True or False: The assessment target '${assessment}' can be met without using any specific lesson content.`, answer: "False", points: 1 },
    { number: 7, type: "fill-in-blank", question: `Fill in the blank: A strong response to ${lesson.title} should include accurate vocabulary and specific __________.`, answer: "evidence or reasoning", points: 1 },
    { number: 8, type: "short-answer", question: "Identify one likely misconception from this lesson and explain the correction.", answer: "A strong answer names a lesson-specific misconception and gives the accurate correction.", points: 3, rubric: "Full credit (3 pts): specific misconception and accurate correction. Partial credit (1-2 pts): vague but related. No credit: off-topic or missing." },
    { number: 9, type: "multiple-choice", question: `Which response best shows understanding of ${lesson.title}?`, options: ["A) A response that uses lesson vocabulary, evidence, and reasoning", "B) A response based only on personal preference", "C) A response about a different course", "D) A response that copies words without explaining them"], answer: "A) A response that uses lesson vocabulary, evidence, and reasoning", points: 2 },
  ];
  fs.writeFileSync(path.join(outDir, `${lesson.id}-quiz-replacement.json`), JSON.stringify({
    id: `${lesson.id}-quiz`,
    lessonId: lesson.id,
    title: `${lesson.title} - Quiz`,
    grade: lesson.grade,
    subject: lesson.subject,
    type: "summative-quiz",
    totalPoints: 20,
    timeAllowed: lesson.duration,
    questions,
  }, null, 2));
}

let lessons = 0;
let files = 0;
for (const grade of grades) {
  const sourceRoot = `D:/lesson-plans/${grade}`;
  const outputRoot = `D:/lesson-plans/additional-materials/${grade}`;
  const lessonFiles = [];
  function walk(dir) {
    for (const name of fs.readdirSync(dir)) {
      const file = path.join(dir, name);
      if (fs.statSync(file).isDirectory()) walk(file);
      else if (name.endsWith(".json") && !name.endsWith("-quiz.json")) lessonFiles.push(file);
    }
  }
  walk(sourceRoot);
  for (const lessonFile of lessonFiles) {
    const lesson = readLesson(lessonFile);
    const relativeDir = path.relative(sourceRoot, path.dirname(lessonFile));
    const outDir = path.join(outputRoot, relativeDir, lesson.id);
    fs.mkdirSync(outDir, { recursive: true });
    writeContentBrief(lesson, outDir);
    writePractice(lesson, outDir);
    writeVocabulary(lesson, outDir);
    writeStudyGuide(lesson, outDir);
    writeExitTicket(lesson, outDir);
    writeQuiz(lesson, outDir);
    lessons += 1;
    files += 6;
  }
}

console.log(`Generated ${files} supplemental files for ${lessons} high school lessons.`);
