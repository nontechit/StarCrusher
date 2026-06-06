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
    const sectionMatch = raw.match(new RegExp(`"${section}"\\s*:\\s*\\{([\\s\\S]*?)(?:"wrapUp"|"assessment"|"differentiation"|$)`));
    if (!sectionMatch) return [];
    const stepsMatch = sectionMatch[1].match(/"steps"\s*:\s*\[([\s\S]*?)\]/);
    if (!stepsMatch) return [];
    return [...stepsMatch[1].matchAll(/"([^"]+)"/g)].map((m) => m[1].replace(/\s+/g, " ").trim());
  };
  const vocab = [];
  for (const match of raw.matchAll(/"term"\s*:\s*"([^"]+)"[\s\S]{0,80}?(?:"definition"|Definition")\s*:\s*"([^"]+)"/g)) {
    vocab.push({ term: match[1].trim(), definition: match[2].trim() });
  }
  const id = prop("id") || path.basename(file, ".json");
  const title = prop("title") || `${id} Lesson`;
  const grade = prop("grade") || (file.split(/[\\/]/).includes("Senior") ? "Senior" : "");
  const subject = prop("subject") || path.basename(path.dirname(file));
  return {
    id,
    title,
    grade,
    subject,
    duration: prop("duration"),
    prerequisites: arrayProp("prerequisites"),
    objectives: arrayProp("objectives"),
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
  };
}

function clean(text) {
  return String(text || "")
    .replace(/^(Teach|Model|Introduce|Guided practice|Students|Small group|Real-world contexts|Ask|Discuss|Read|Present|Show|Pose|Set|Connect|Review|Quick check|Preview|Teacher model|Independent practice|Guided practice):\s*/i, "")
    .replace(/\s+/g, " ")
    .replace(/[.]+$/, "")
    .trim();
}

function optionText(text) {
  const value = clean(text);
  return value.length > 210 ? `${value.slice(0, 207)}...` : value;
}

function subjectApplication(lesson, step) {
  const subject = String(lesson.subject || "");
  if (subject.includes("Math")) return `Apply the method from this lesson activity to solve a related problem: ${step}`;
  if (subject.includes("Science")) return `Use the scientific idea from this lesson activity to explain a related observation or data pattern: ${step}`;
  if (subject.includes("Social")) return `Use the historical, civic, geographic, or economic idea from this activity to explain a related situation: ${step}`;
  if (subject.includes("World") || subject.includes("Language")) return `Use the language skill from this lesson activity in a related communication task: ${step}`;
  if (subject.includes("Elective")) return `Apply the career, arts, wellness, technology, or life skill from this activity to a realistic scenario: ${step}`;
  return `Use the reading, writing, speaking, or analysis skill from this activity to respond to a related text or task: ${step}`;
}

function makeQuiz(lesson, previousQuiz = {}) {
  const vocab = lesson.vocabulary || [];
  const term1 = vocab[0]?.term || "key concept";
  const def1 = vocab[0]?.definition || "an important idea from the lesson";
  const term2 = vocab[1]?.term || term1;
  const def2 = vocab[1]?.definition || def1;
  const objective1 = lesson.objectives?.[0] || `Students will explain ${lesson.title}`;
  const objective2 = lesson.objectives?.[1] || objective1;
  const mainSteps = lesson.procedure?.mainActivity?.steps || [];
  const activity1 = optionText(mainSteps[0] || lesson.procedure?.warmup?.steps?.[0] || lesson.title);
  const activity2 = optionText(mainSteps[1] || mainSteps[0] || lesson.title);
  const activity3 = optionText(mainSteps[2] || mainSteps[1] || mainSteps[0] || lesson.title);
  const prior = optionText(lesson.prerequisites?.[0] || "prior course skills");
  const assessment = optionText(lesson.assessment?.summative?.[0] || objective1);
  const duration = lesson.duration || previousQuiz.timeAllowed;

  return {
    id: `${lesson.id}-quiz`,
    lessonId: lesson.id,
    title: previousQuiz.title && !/Assessment Quiz$/.test(previousQuiz.title)
      ? previousQuiz.title
      : `${lesson.title} Assessment Quiz`,
    grade: lesson.grade || previousQuiz.grade,
    subject: lesson.subject || previousQuiz.subject,
    type: previousQuiz.type || "formative-quiz",
    ...(duration ? { timeAllowed: duration } : {}),
    questions: [
      {
        number: 1,
        type: "multiple-choice",
        question: `In ${lesson.title}, what does '${term1}' mean?`,
        options: [
          `A) ${def1}`,
          "B) A review topic from an unrelated course",
          "C) A personal preference that does not require evidence",
          "D) A future unit topic not used in this lesson",
        ],
        answer: "A",
        points: 1,
      },
      {
        number: 2,
        type: "multiple-choice",
        question: `Which activity detail is part of ${lesson.id}: ${lesson.title}?`,
        options: [
          `A) ${activity1}`,
          "B) Students complete an unrelated memorization drill",
          "C) Students skip the lesson skill and only copy definitions",
          "D) Students work on a topic from a different course",
        ],
        answer: "A",
        points: 1,
      },
      {
        number: 3,
        type: "short-answer",
        question: `Explain how the activity '${activity2}' helps students meet this objective: '${objective1}'.`,
        answer: `A complete response connects the activity to '${objective1}' and uses a specific detail from the lesson activity.`,
        points: 2,
        rubric: "Full credit (2 pts): clearly connects the activity to the objective and includes a specific lesson detail. Partial credit (1 pt): names the activity or objective but gives an incomplete explanation. No credit: off-topic or missing.",
      },
      {
        number: 4,
        type: "short-answer",
        question: `Define '${term2}' and use it correctly in a sentence connected to ${lesson.title}.`,
        answer: `A correct response defines '${term2}' as '${def2}' and uses the term accurately in lesson context.`,
        points: 2,
        rubric: "Full credit (2 pts): accurate definition and context-appropriate sentence. Partial credit (1 pt): definition or sentence is partly correct. No credit: inaccurate or missing.",
      },
      {
        number: 5,
        type: "application",
        question: subjectApplication(lesson, activity3),
        answer: "A complete response applies the lesson skill or concept accurately and explains the reasoning with details from the activity.",
        points: 2,
        rubric: "Full credit (2 pts): accurate application with clear reasoning and a lesson-specific detail. Partial credit (1 pt): partly accurate application or limited explanation. No credit: off-topic or unsupported.",
      },
      {
        number: 6,
        type: "analysis",
        question: `This lesson builds on '${prior}' and assesses '${assessment}'. What misconception might interfere with that learning, and how should it be corrected?`,
        answer: "A strong response identifies a plausible misconception tied to the lesson content and explains the accurate correction.",
        points: 2,
        rubric: "Full credit (2 pts): identifies a lesson-specific misconception and provides an accurate correction. Partial credit (1 pt): misconception or correction is vague. No credit: off-topic or missing.",
      },
    ],
    totalPoints: 10,
    scoringGuide: previousQuiz.scoringGuide || "Exceeds (90-100%): 9-10 pts. Proficient (70-89%): 7-8 pts. Developing (50-69%): 5-6 pts. Below Standard: under 5 pts.",
    teacherNotes: previousQuiz.teacherNotes || "May be administered as a written formative assessment, exit ticket, or orally with accommodations. Use student responses to guide reteaching decisions.",
  };
}

let written = 0;
let skipped = 0;
for (const grade of grades) {
  const root = `D:/lesson-plans/${grade}`;
  const lessonFiles = [];
  function walk(dir) {
    for (const name of fs.readdirSync(dir)) {
      const file = path.join(dir, name);
      if (fs.statSync(file).isDirectory()) walk(file);
      else if (name.endsWith(".json") && !name.endsWith("-quiz.json")) lessonFiles.push(file);
    }
  }
  walk(root);
  for (const lessonFile of lessonFiles) {
    let lesson;
    try {
      lesson = readJson(lessonFile);
    } catch {
      lesson = parseLooseLesson(lessonFile);
    }
    const quizFile = lessonFile.replace(/\.json$/, "-quiz.json");
    let previousQuiz = {};
    if (fs.existsSync(quizFile)) {
      try {
        previousQuiz = readJson(quizFile);
      } catch {
        previousQuiz = {};
      }
    }
    const quiz = makeQuiz(lesson, previousQuiz);
    fs.writeFileSync(quizFile, JSON.stringify(quiz, null, 2));
    written += 1;
  }
}

console.log(`Updated ${written} high school source quizzes. Skipped ${skipped} malformed lesson files.`);
