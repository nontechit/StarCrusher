import fs from "node:fs";
import path from "node:path";

const grades = ["Sixth", "Seventh"];
const subjects = ["English-Language-Arts", "Mathematics", "Science", "Social-Studies"];

function readJson(file) {
  const raw = fs.readFileSync(file, "utf8").replace(/^\uFEFF/, "").trim();
  try {
    return JSON.parse(raw);
  } catch {
    let depth = 0;
    let inString = false;
    let escaped = false;
    let start = raw.indexOf("{");
    for (let i = start; i < raw.length; i++) {
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
      if (ch === "{") depth++;
      if (ch === "}") depth--;
      if (depth === 0) return JSON.parse(raw.slice(start, i + 1));
    }
    let fixed = raw;
    const opens = (fixed.match(/{/g) || []).length;
    let closes = (fixed.match(/}/g) || []).length;
    while (closes < opens) {
      fixed += "}";
      closes++;
    }
    return JSON.parse(fixed);
  }
}

function compact(text) {
  return String(text || "")
    .replace(/^(Teach|Model|Introduce|Guided practice|Students|Small group|Real-world contexts|Ask|Discuss|Read|Present|Show|Pose|Set|Connect|Review|Quick check|Preview|Class discussion|Exit ticket):\s*/i, "")
    .replace(/\([^)]{80,}\)/g, "")
    .replace(/\s+/g, " ")
    .replace(/[.]+$/, "")
    .trim();
}

function lessonDetail(lesson) {
  return compact(
    lesson.procedure?.mainActivity?.steps?.[1] ||
    lesson.procedure?.mainActivity?.steps?.[0] ||
    lesson.procedure?.warmup?.steps?.[0] ||
    lesson.title
  );
}

function definitionPhrase(definition) {
  return compact(definition)
    .replace(/^[Tt]he /, "")
    .replace(/^[Aa]n? /, "")
    .replace(/;.*$/, "")
    .toLowerCase();
}

const exact = new Map(Object.entries({
  "textual evidence": "The writer quoted a specific sentence from the story as textual evidence for the theme.",
  "character arc": "The character arc shows Maya changing from afraid to confident by the end of the novel.",
  "central idea": "The central idea of the article is that clean water depends on both technology and community action.",
  "text structure": "The cause-and-effect text structure helps readers see why the river became polluted.",
  "author's claim": "The author's claim is that students should have more outdoor learning time.",
  "credible evidence": "Recent data from a reliable science report is credible evidence for the argument.",
  "logical reasoning": "Her logical reasoning connects the survey results to the conclusion without skipping steps.",
  "domain-specific vocabulary": "Words like numerator, denominator, and quotient are domain-specific vocabulary in math.",
  "narrative voice": "The narrator's sarcastic narrative voice makes the scene feel funny instead of frightening.",
  "pacing": "Short sentences quicken the pacing during the chase scene.",
  "tension": "The locked door and ticking clock create tension before the character escapes.",
  "counterclaim": "The counterclaim says school uniforms limit student expression.",
  "rebuttal": "The rebuttal explains why the opposing argument is not strong enough.",
  "synthesize": "Readers synthesize two articles when they combine ideas from both into one explanation.",
  "paraphrase": "To paraphrase the paragraph, Lena restated the author's idea in her own words.",
  "citation": "The citation tells readers exactly where the quoted evidence came from.",
  "appositive": "In the sentence 'Ms. Rivera, our coach, smiled,' the phrase 'our coach' is an appositive.",
  "semicolon": "A semicolon can join two closely related complete sentences.",
  "dash": "The dash adds a sudden explanation at the end of the sentence.",
  "syntax": "The poem's unusual syntax places the verb first to emphasize the action.",
  "connotation": "The word 'home' has a warm connotation for many readers.",
  "denotation": "The denotation of 'slender' is thin, but its feeling is often positive.",
  "academic vocabulary": "Words such as analyze, evaluate, and compare are academic vocabulary used across subjects.",
  "nuance": "The nuance between 'angry' and 'furious' shows a difference in intensity.",
  "credibility": "A source gains credibility when its facts are accurate and its author is trustworthy.",
  "bias": "The advertisement shows bias because it only presents facts that make the product look good.",
  "primary source": "A diary written during the event is a primary source.",
  "MLA format": "In MLA format, the student's paper includes a Works Cited page.",
  "Socratic seminar": "During the Socratic seminar, students built on one another's ideas with text evidence.",
  "active listening": "Active listening means looking at the speaker and responding to the actual idea shared.",
  "elaboration": "The second sentence adds elaboration by explaining how the quote supports the claim.",
  "accountable talk": "Accountable talk includes phrases like 'I agree because' and 'What evidence supports that?'",
  "stanza": "Each stanza in the poem develops a different image of winter.",
  "imagery": "The phrase 'silver rain on the dark roof' creates imagery the reader can picture.",
  "free verse": "The free verse poem does not follow a regular rhyme scheme or meter.",
  "alliteration": "The phrase 'silent silver snow' uses alliteration.",
  "enjambment": "Enjambment carries one line of poetry into the next without a pause.",
  "media": "A video, podcast, poster, or article can all be forms of media.",
  "audience": "The audience for the school flyer is students and their families.",
  "propaganda": "The poster uses propaganda by exaggerating facts to influence people's opinions.",
  "framing": "The photograph's framing makes the speaker look powerful by placing her above the crowd.",
  "book club": "In book club, each student discussed a different chapter question.",
  "literary analysis": "Her literary analysis explains how the symbol of the storm reveals the character's fear.",
  "reading identity": "A student's reading identity includes the genres, authors, and topics they enjoy.",
  "recommendation": "His recommendation explains why mystery fans would enjoy the novel.",
  "ratio": "The ratio of red marbles to blue marbles is 3 to 5.",
  "rate": "Driving 120 miles in 2 hours is a rate of 60 miles per hour.",
  "unit rate": "A price of $12 for 4 tickets has a unit rate of $3 per ticket.",
  "equivalent ratio": "The ratios 2:3 and 4:6 are equivalent ratios because they describe the same relationship.",
  "percent": "A percent compares a number to 100, so 25% means 25 out of 100.",
  "proportion": "The proportion 2/3 = 8/12 shows two equal ratios.",
  "discount": "A 20% discount lowers the original price of the jacket.",
  "tax": "The sales tax adds a small percent to the cost of the notebook.",
  "integer": "Negative 4, zero, and 7 are all integers.",
  "negative number": "A negative number can represent a temperature below zero.",
  "absolute value": "The absolute value of -8 is 8 because it is 8 units from zero.",
  "opposite": "The opposite of 6 is -6 on the number line.",
  "reciprocal": "The reciprocal of 3/4 is 4/3.",
  "dividend": "In 42 divided by 6, 42 is the dividend.",
  "divisor": "In 42 divided by 6, 6 is the divisor.",
  "quotient": "The quotient of 42 divided by 6 is 7.",
  "expression": "The expression 4x + 3 has a variable, a coefficient, and a constant.",
  "coefficient": "In 7x, the coefficient is 7.",
  "constant": "In x + 9, the constant is 9 because its value does not change.",
  "term": "In 5x + 2, both 5x and 2 are terms.",
  "equation": "The equation x + 4 = 10 is true when x equals 6.",
  "solution": "The solution to y - 3 = 8 is y = 11.",
  "inequality": "The inequality x > 5 means x can be any number greater than 5.",
  "coordinate plane": "The coordinate plane uses an x-axis and a y-axis to locate points.",
  "ordered pair": "The ordered pair (3, 2) means move 3 units right and 2 units up.",
  "x-axis": "The x-axis runs horizontally across the coordinate plane.",
  "y-axis": "The y-axis runs vertically across the coordinate plane.",
  "quadrant": "Point (-2, 5) is in Quadrant II.",
  "area": "The area of the rectangle is the number of square units inside it.",
  "surface area": "Surface area measures the total area of all faces of a three-dimensional figure.",
  "volume": "Volume tells how many cubic units fill a prism.",
  "statistical question": "How many books did each student read this month is a statistical question.",
  "data": "The class collected data by measuring each plant's height every day.",
  "mean": "The mean is found by adding the values and dividing by the number of values.",
  "median": "The median is the middle value when the data are in order.",
  "mode": "The mode is the value that appears most often.",
  "range": "The range is the difference between the greatest and least values.",
  "dot plot": "A dot plot uses dots above a number line to show how often values occur.",
  "histogram": "A histogram groups numerical data into intervals.",
  "box plot": "A box plot shows the median, quartiles, and range of a data set.",
  "scientific inquiry": "Scientific inquiry begins with a question that can be investigated through evidence.",
  "hypothesis": "Her hypothesis was, 'If the water is warmer, then the sugar will dissolve faster.'",
  "variable": "The amount of sunlight was the variable changed in the plant experiment.",
  "metric system": "Scientists use the metric system when they measure length in meters and mass in grams.",
  "qualitative observation": "Saying the rock is rough and gray is a qualitative observation.",
  "quantitative observation": "Saying the rock has a mass of 42 grams is a quantitative observation.",
  "atom": "An atom is the tiny building block that makes up matter.",
  "element": "Oxygen is an element because it contains only one kind of atom.",
  "molecule": "A water molecule has two hydrogen atoms and one oxygen atom.",
  "compound": "Water is a compound made from hydrogen and oxygen chemically joined together.",
  "physical property": "Color is a physical property because it can be observed without changing the substance.",
  "chemical property": "Flammability is a chemical property because it describes how a substance can react.",
  "mixture": "Trail mix is a mixture because its parts are combined but not chemically changed.",
  "solution": "Salt water is a solution because the salt dissolves evenly in the water.",
  "solvent": "In salt water, water is the solvent because it dissolves the salt.",
  "solute": "In salt water, salt is the solute because it gets dissolved.",
  "concentration": "A drink with more powder mixed into the same amount of water has a higher concentration.",
  "energy": "Energy allows a ball to move, a light to shine, or water to heat up.",
  "kinetic energy": "A rolling skateboard has kinetic energy because it is moving.",
  "potential energy": "A book on a high shelf has potential energy because of its position.",
  "thermal energy": "Hot soup has more thermal energy than cold soup.",
  "conduction": "Conduction warms a metal spoon when it touches hot soup.",
  "convection": "Convection moves warm air upward and cooler air downward.",
  "radiation": "Radiation from the Sun warms Earth's surface.",
  "force": "A force is a push or pull that can change an object's motion.",
  "motion": "The car's motion changed when it sped up around the curve.",
  "speed": "Speed tells how far an object travels in a certain amount of time.",
  "velocity": "Velocity includes both speed and direction, such as 20 meters per second north.",
  "acceleration": "Acceleration happens when an object speeds up, slows down, or changes direction.",
  "gravity": "Gravity pulls objects toward Earth's center.",
  "friction": "Friction between the tires and road helps the bike stop.",
  "ecosystem": "A pond ecosystem includes water, plants, fish, insects, sunlight, and soil.",
  "population": "A population is all the deer living in the same forest.",
  "community": "A community includes all the different populations living in one ecosystem.",
  "habitat": "A cactus's desert habitat provides the light, soil, and water it needs.",
  "food chain": "A food chain can show grass being eaten by a rabbit and the rabbit being eaten by a fox.",
  "food web": "A food web shows many connected feeding relationships in an ecosystem.",
  "producer": "A producer such as grass makes its own food using sunlight.",
  "consumer": "A consumer such as a rabbit gets energy by eating other organisms.",
  "decomposer": "A decomposer such as a mushroom breaks down dead matter.",
  "adaptation": "Thick fur is an adaptation that helps an animal survive cold weather.",
  "weather": "Weather describes today's temperature, wind, clouds, and precipitation.",
  "climate": "Climate describes the usual weather patterns of a place over many years.",
  "atmosphere": "Earth's atmosphere is the layer of gases surrounding the planet.",
  "greenhouse effect": "The greenhouse effect keeps Earth warm enough for life by trapping some heat.",
  "fossil fuel": "Coal, oil, and natural gas are fossil fuels formed from ancient living things.",
  "renewable resource": "Sunlight is a renewable resource because it is naturally replaced quickly.",
  "nonrenewable resource": "Oil is a nonrenewable resource because it takes millions of years to form.",
  "sustainability": "Sustainability means using resources in a way that people in the future can still meet their needs.",
  "pollution": "Pollution enters a river when harmful chemicals or trash contaminate the water.",
  "biodiversity": "A rainforest has high biodiversity because many different species live there.",
  "natural resource": "Fresh water is a natural resource that people use for drinking and farming.",
  "conservation": "Turning off unused lights is one example of conservation.",
  "habitat loss": "Habitat loss occurs when forests are cleared and animals lose the places they need to live.",
  "ecological footprint": "A person's ecological footprint grows when they use more energy, water, and materials.",
  "evidence": "The evidence included measurements, observations, or source details that supported the conclusion.",
  "system": "A system is easier to understand when each part and its job are shown clearly.",
  "economy": "The economy grew when farmers, merchants, and workers produced and traded more goods.",
  "natural resources": "Natural resources such as fertile soil and fresh water helped the region support farming.",
  "trade": "Trade allowed people in different regions to exchange goods they could not produce themselves.",
  "import": "Coffee was an import because it was brought into the country from another place.",
  "export": "Wheat was an export because farmers sold it to buyers in other countries.",
  "developed country": "A developed country usually has strong infrastructure, advanced industry, and a high standard of living.",
  "developing country": "A developing country may be building stronger roads, schools, industries, and health systems.",
  "interdependence": "Interdependence means countries rely on one another for resources, goods, and services.",
  "physical change": "Melting ice is a physical change because the water changes form but stays the same substance.",
  "chemical change": "Burning wood is a chemical change because new substances form.",
  "conservation of mass": "Conservation of mass means the total mass before and after a reaction stays the same.",
  "precipitate": "A yellow precipitate formed when the two clear liquids were mixed.",
  "argument": "An argument includes a claim supported by reasons and evidence.",
  "faulty reasoning": "Faulty reasoning appears when a conclusion does not logically follow from the evidence.",
  "rhetorical appeal": "A rhetorical appeal uses logic, emotion, or credibility to persuade an audience.",
  "counterargument": "A counterargument explains the opposing side of an issue.",
  "thesis statement": "The thesis statement clearly states the main argument of the essay.",
  "integration": "Integration of a quote means blending it smoothly into the writer's sentence.",
  "signal phrase": "The signal phrase 'According to the author' introduces quoted evidence.",
  "internal conflict": "Internal conflict occurs when a character struggles with fear, guilt, or a hard choice.",
  "external conflict": "External conflict occurs when a character faces another person, nature, society, or an outside force.",
  "characterization": "The author uses characterization when showing that the character is brave through actions and dialogue.",
  "subtext": "The subtext of the scene suggests the character is jealous, even though she says she is fine.",
  "round character": "A round character has complex traits and can surprise the reader.",
  "academic voice": "Academic voice sounds clear, formal, and focused on evidence.",
  "coherence": "Coherence helps each sentence connect smoothly to the next.",
  "transition": "The transition 'however' signals a contrast between ideas.",
  "precision": "Precision improves writing when a student changes 'stuff' to 'renewable resources.'",
  "participial phrase": "In 'Running down the hall, Mia dropped her books,' the words 'Running down the hall' form a participial phrase.",
  "infinitive phrase": "In 'to solve the problem,' the words form an infinitive phrase.",
  "phrase": "A phrase is a group of related words that does not have both a subject and a verb.",
  "clause": "A clause is a group of words with a subject and a verb.",
  "etymology": "Etymology helps explain that 'biology' comes from roots meaning life and study.",
  "word family": "The words predict, prediction, and predictable belong to the same word family.",
  "cognate": "The English word family and the Spanish word familia are cognates.",
  "semantic change": "Semantic change happens when a word's meaning shifts over time.",
  "cross-examination": "During cross-examination, one speaker asks questions to test the other side's evidence.",
  "concession": "A concession admits that part of the opposing argument has value.",
  "volta": "The volta is the turn in a poem where the speaker's thought or feeling shifts.",
  "extended metaphor": "The poem's extended metaphor compares life to a long road across several lines.",
  "social commentary": "The story offers social commentary by criticizing unfair treatment in the community.",
  "tone": "The author's tone is hopeful when she describes the community rebuilding together.",
  "annotated bibliography": "An annotated bibliography lists sources and briefly explains how each one is useful.",
  "synthesis": "Synthesis combines ideas from several sources into one new understanding.",
  "thesis": "The thesis states the essay's main idea in one focused sentence.",
  "multimedia": "A multimedia presentation can combine narration, images, charts, and video.",
  "documentary": "A documentary uses real footage, interviews, and narration to explain a topic.",
  "film technique": "A close-up is a film technique that draws attention to a character's emotion.",
  "juxtaposition": "Juxtaposition places two contrasting images side by side to emphasize a difference.",
  "narration": "The narration explains events that the viewer cannot see on screen.",
  "critical reading": "Critical reading means questioning the author's choices, evidence, and purpose.",
  "literary criticism": "Literary criticism explains and evaluates how a text creates meaning.",
  "review": "A review evaluates a book, film, or performance and supports an opinion with reasons.",
  "rational number": "A rational number can be written as a fraction, such as -3/4 or 5.",
  "additive inverse": "The additive inverse of 9 is -9 because their sum is zero.",
  "multiplicative inverse": "The multiplicative inverse of 2/3 is 3/2 because their product is 1.",
  "proportional relationship": "A proportional relationship has equivalent ratios and passes through the origin on a graph.",
  "constant of proportionality": "In y = 4x, the constant of proportionality is 4.",
  "direct variation": "Direct variation means one quantity changes by a constant multiple of another.",
  "percent increase": "A price rising from $20 to $25 is a percent increase.",
  "percent decrease": "A price dropping from $50 to $40 is a percent decrease.",
  "simple interest": "Simple interest is money earned or paid based on the principal, rate, and time.",
  "markup": "A store adds markup when it raises the wholesale price to make a selling price.",
  "two-step equation": "The two-step equation 2x + 3 = 11 can be solved by subtracting 3 and then dividing by 2.",
  "multi-step": "A multi-step problem requires more than one operation to solve.",
  "inverse operations": "Addition and subtraction are inverse operations because they undo each other.",
  "scale drawing": "A scale drawing uses a constant ratio to represent a larger or smaller object.",
  "scale factor": "A scale factor of 3 makes each length three times as large.",
  "scale": "On a map, the scale might show that 1 inch represents 10 miles.",
  "vertical angles": "Vertical angles are opposite angles formed when two lines cross.",
  "complementary angles": "Complementary angles have measures that add to 90 degrees.",
  "supplementary angles": "Supplementary angles have measures that add to 180 degrees.",
  "circumference": "Circumference is the distance around a circle.",
  "diameter": "The diameter passes through the center of a circle from one side to the other.",
  "radius": "The radius reaches from the center of a circle to the edge.",
  "probability": "Probability describes how likely an event is to happen.",
  "sample space": "The sample space for flipping a coin is heads and tails.",
  "compound event": "Rolling an even number and flipping heads is a compound event.",
  "simulation": "A simulation can model 100 coin flips without flipping a real coin each time.",
  "organism": "A mushroom is an organism because it is an individual living thing.",
  "homeostasis": "Sweating helps the body maintain homeostasis when it gets too hot.",
  "classification": "Classification groups organisms by shared traits.",
  "taxonomy": "Taxonomy is the science scientists use to name and classify organisms.",
  "binomial nomenclature": "Binomial nomenclature gives each species a two-part scientific name, such as Homo sapiens.",
  "cell": "A cell is the basic unit that makes up living things.",
  "cell membrane": "The cell membrane controls what enters and leaves the cell.",
  "nucleus": "The nucleus contains genetic information and helps control the cell.",
  "cytoplasm": "Cytoplasm is the jelly-like material inside a cell.",
  "mitochondria": "Mitochondria release energy from food for the cell.",
  "chloroplast": "A chloroplast captures sunlight for photosynthesis in plant cells.",
  "cell wall": "A cell wall gives a plant cell extra support and shape.",
  "photosynthesis": "During photosynthesis, plants use sunlight, water, and carbon dioxide to make sugar.",
  "cellular respiration": "Cellular respiration releases energy from sugar inside cells.",
  "glucose": "Glucose is a sugar that cells can use for energy.",
  "chlorophyll": "Chlorophyll is the green pigment that absorbs sunlight in leaves.",
  "respiration": "Respiration uses oxygen to release energy from food.",
  "circulatory system": "The circulatory system moves blood, oxygen, and nutrients through the body.",
  "respiratory system": "The respiratory system brings oxygen into the body and removes carbon dioxide.",
  "digestive system": "The digestive system breaks food into nutrients the body can absorb.",
  "nervous system": "The nervous system sends messages between the brain and the rest of the body.",
  "organ": "The heart is an organ that pumps blood.",
  "tissue": "Muscle tissue is made of similar cells that work together to move the body.",
  "organ system": "An organ system is a group of organs that work together to do a major job.",
  "immune system": "The immune system protects the body from germs and disease.",
  "pathogen": "A pathogen such as a virus or bacterium can cause disease.",
  "antibody": "An antibody helps the immune system recognize and fight a specific pathogen.",
  "vaccine": "A vaccine trains the immune system to recognize a disease without causing the illness.",
  "virus": "A virus needs a host cell to reproduce.",
  "bacteria": "Some bacteria help digestion, while others can cause disease.",
  "antibiotic": "An antibiotic can kill bacteria but does not work on viruses.",
  "genetics": "Genetics explains how traits pass from parents to offspring.",
  "gene": "A gene is a section of DNA that carries instructions for a trait.",
  "chromosome": "A chromosome is a long strand of DNA found in the cell nucleus.",
  "DNA": "DNA carries genetic instructions for living things.",
  "trait": "Eye color is an example of an inherited trait.",
  "dominant": "A dominant allele can show its trait even when only one copy is present.",
  "recessive": "A recessive allele shows its trait only when two copies are present.",
  "genotype": "The genotype Bb shows the alleles an organism has for a trait.",
  "phenotype": "The phenotype is the visible trait, such as purple flowers.",
  "Punnett square": "A Punnett square predicts possible allele combinations in offspring.",
  "evolution": "Evolution is the change in inherited traits of populations over many generations.",
  "natural selection": "Natural selection can make helpful traits more common in a population.",
  "variation": "Variation means individuals in a population have different traits.",
  "fossil": "A fossil can preserve evidence of an organism that lived long ago.",
  "extinction": "Extinction happens when every member of a species has died.",
  "geologic time": "Geologic time describes Earth's history across billions of years.",
  "plate tectonics": "Plate tectonics explains how Earth's outer plates move and interact.",
  "lithosphere": "The lithosphere includes Earth's crust and the rigid upper mantle.",
  "asthenosphere": "The asthenosphere is the softer layer that tectonic plates move over.",
  "convergent boundary": "At a convergent boundary, plates move toward each other.",
  "divergent boundary": "At a divergent boundary, plates move apart.",
  "transform boundary": "At a transform boundary, plates slide past each other.",
  "earthquake": "An earthquake happens when stored energy is suddenly released along a fault.",
  "volcano": "A volcano can form where magma reaches Earth's surface.",
  "rock cycle": "The rock cycle shows how rocks change from one type to another over time.",
  "igneous rock": "Igneous rock forms when melted rock cools and hardens.",
  "sedimentary rock": "Sedimentary rock forms when layers of sediment are compacted and cemented.",
  "metamorphic rock": "Metamorphic rock forms when heat and pressure change existing rock.",
  "weathering": "Weathering breaks rock into smaller pieces.",
  "erosion": "Erosion moves sediment by wind, water, ice, or gravity.",
  "deposition": "Deposition happens when sediment is dropped in a new place.",
  "water cycle": "The water cycle moves water through evaporation, condensation, precipitation, and collection.",
  "evaporation": "Evaporation changes liquid water into water vapor.",
  "condensation": "Condensation forms clouds when water vapor cools into tiny droplets.",
  "precipitation": "Rain, snow, sleet, and hail are forms of precipitation.",
  "runoff": "Runoff flows over land into streams, rivers, and lakes.",
  "groundwater": "Groundwater is water stored underground in spaces between rocks and soil.",
  "watershed": "A watershed is the land area that drains into the same body of water.",
  "aquifer": "An aquifer is an underground layer that stores and carries water.",
  "strategy": "One useful strategy is to choose a model or step-by-step method before solving the problem.",
  "justify": "Students justify an answer when they explain why their method or evidence proves it is correct.",
  "model": "A model such as a table, diagram, equation, map, or chart can represent an idea clearly.",
  "reasonable": "An answer is reasonable when it fits the numbers, evidence, or situation.",
  "inference": "The reader made an inference by combining evidence with what the details suggested.",
  "claim": "The paragraph began with a claim that stated the writer's position.",
  "context": "The context around the unfamiliar word showed that it meant a serious warning.",
  "interpretation": "The student's interpretation explained what the details suggested.",
  "source": "A source such as a map, text, image, or artifact gave students information to analyze.",
  "perspective": "A person's perspective shaped how they understood the event or issue."
}));

function sentenceFor(term, definition, lesson) {
  if (exact.has(term)) return exact.get(term);
  const def = definitionPhrase(definition);
  const startsUpper = /^[A-Z]/.test(term);
  if (lesson.subject === "Mathematics") {
    return `Students used the ${term} to represent ${def} in the math problem.`;
  }
  if (lesson.subject === "Science") {
    return `The diagram labeled the ${term} as ${def}.`;
  }
  if (String(lesson.subject).includes("Social")) {
    if (startsUpper) return `${term} shaped the history, culture, or geography studied in the lesson.`;
    return `The ${term} was important because it involved ${def}.`;
  }
  return `The writer discussed ${term} while analyzing ${def}.`;
}

for (const grade of grades) {
  for (const subject of subjects) {
    const lessonDir = `D:/lesson-plans/${grade}/${subject}`;
    const vocabRoot = `D:/lesson-plans/additional-materials/${grade}/${subject}`;
    const lessonFiles = fs.readdirSync(lessonDir)
      .filter((name) => name.endsWith(".json") && !name.endsWith("-quiz.json"))
      .sort();
    for (const lessonFile of lessonFiles) {
      const lesson = readJson(path.join(lessonDir, lessonFile));
      const vocabFile = path.join(vocabRoot, lesson.id, "vocabulary-builder.json");
      const vocab = readJson(vocabFile);
      for (const entry of vocab.vocabulary) {
        entry.exampleSentence = sentenceFor(entry.term, entry.definition, lesson);
      }
      fs.writeFileSync(vocabFile, JSON.stringify(vocab, null, 2));
    }
  }
}

console.log("Updated vocabulary example sentences for Sixth and Seventh.");
