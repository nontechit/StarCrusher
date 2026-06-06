import fs from "node:fs";

const updates = new Map([
  ["D:/lesson-plans/additional-materials/Kindergarten/English-Language-Arts/K-ELA-09/vocabulary-builder.json::adjective", "The word blue is an adjective because it describes the ball."],
  ["D:/lesson-plans/additional-materials/Kindergarten/English-Language-Arts/K-ELA-09/vocabulary-builder.json::sentence", "The words 'I see a red ball' make a complete sentence."],
  ["D:/lesson-plans/additional-materials/Kindergarten/English-Language-Arts/K-ELA-10/vocabulary-builder.json::retell", "I can retell the story: first the frog jumped, then it splashed, then it hopped home."],
  ["D:/lesson-plans/additional-materials/Kindergarten/Mathematics/K-MATH-01/vocabulary-builder.json::teen numbers", "Teen numbers like 15 have 1 ten and some more ones."],
  ["D:/lesson-plans/additional-materials/Kindergarten/Mathematics/K-MATH-06/vocabulary-builder.json::property", "One property of a triangle is that it has 3 sides."],
  ["D:/lesson-plans/additional-materials/Kindergarten/Science/K-SCI-10/vocabulary-builder.json::property", "Color is one property we used to describe and sort the rocks."],
  ["D:/lesson-plans/additional-materials/Kindergarten/Social-Studies/K-SOC-05/vocabulary-builder.json::services", "Haircuts and bus rides are services because people do work to help others."],
  ["D:/lesson-plans/additional-materials/Second/Art-Music-PE/2AMP-01/vocabulary-builder.json::secondary colors", "Orange, green, and purple are secondary colors made by mixing two primary colors."],
  ["D:/lesson-plans/additional-materials/Second/Mathematics/2MATH-01/vocabulary-builder.json::near doubles", "The fact 6 + 7 is one of the near doubles because it is close to 6 + 6."],
]);

for (const [key, sentence] of updates) {
  const [file, term] = key.split("::");
  const raw = fs.readFileSync(file, "utf8");
  const bom = raw.startsWith("\uFEFF") ? "\uFEFF" : "";
  const data = JSON.parse(raw.replace(/^\uFEFF/, "").trim());
  const entry = data.vocabulary.find((item) => item.term === term);
  if (!entry) throw new Error(`Missing term ${term} in ${file}`);
  entry.exampleSentence = sentence;
  fs.writeFileSync(file, bom + JSON.stringify(data, null, 2));
}

console.log(`Updated ${updates.size} lower-grade vocabulary example sentences.`);
