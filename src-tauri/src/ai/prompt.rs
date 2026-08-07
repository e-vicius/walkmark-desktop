use crate::models::Settings;

/// Shared voice and hard rules. Kept in one place so the outline pass and the
/// per-step pass can never drift apart stylistically.
pub fn system_instruction(settings: &Settings, vocabulary: Option<&str>) -> String {
    let mut s = format!(
        "You are a meticulous technical writer. You turn screenshots of somebody \
performing a task into clear, step-by-step product documentation.\n\n\
Write in {language}. Write for {audience}. The voice should be {tone}.\n\n\
Rules you must never break:\n\
- Write instructions, not descriptions. Say \"Click Save\", never \"This screenshot shows the Save button\".\n\
- Address the reader as \"you\". Use the imperative for titles.\n\
- Only refer to text, buttons and labels you can actually read in the screenshot. \
If you cannot read a label, describe the control by position or purpose instead of inventing a name.\n\
- Never invent menu items, keyboard shortcuts, prices, names or results that are not visible.\n\
- Do not number the steps; numbering is added automatically.\n\
- Do not repeat what a previous step already said.\n\
- Ignore the mouse cursor, clocks, battery indicators, notifications and other incidental chrome.\n\
- If a screenshot contains personal data, refer to it generically (\"your email address\") rather than quoting it.\n\
- Never mention screenshots, images, or the fact that you are looking at a picture.",
        language = settings.language.trim(),
        audience = settings.audience.trim(),
        tone = settings.tone.describe(),
    );

    if let Some(glossary) = vocabulary.filter(|s| !s.trim().is_empty()) {
        s.push_str(&format!(
            "\n\nProduct vocabulary you should use (provided by the author):\n{}",
            glossary.trim()
        ));
    }
    s
}

/// Pass 1 — look at every frame at once and decide what the whole task is.
///
/// Doing this before writing any step is what keeps titles consistent and stops
/// the model from narrating the same action three times in a row.
pub fn outline_prompt(count: usize) -> String {
    format!(
        "Below are {count} screenshots taken in chronological order while somebody completed a single task.\n\n\
First work out what the overall task was. Then give each screenshot a short imperative title \
describing the one action the reader should take at that point.\n\n\
Return:\n\
- title: the name of the whole procedure, in title case, at most 70 characters. \
Start with a gerund or noun phrase, for example \"Creating a new invoice\".\n\
- summary: one or two sentences saying what the reader will accomplish and when they would do this.\n\
- prerequisites: up to 4 short items the reader needs before starting. \
Only include ones you can genuinely infer. An empty list is fine and is better than guessing.\n\
- steps: exactly {count} titles, in the same order as the screenshots. \
Each at most 60 characters, imperative, no trailing full stop, no numbering.\n\n\
If two consecutive screenshots show the same action, still give each its own distinct title \
covering the part of the action it shows."
    )
}

/// Pass 2 — write the body for one step, with just enough context to stay
/// coherent without re-sending every frame.
pub fn step_prompt(
    index: usize,
    total: usize,
    task_title: &str,
    planned_title: &str,
    previous: &[String],
) -> String {
    let mut s = String::new();
    if !task_title.is_empty() {
        s.push_str(&format!("Overall task: {task_title}\n"));
    }
    s.push_str(&format!("This is step {} of {}.\n", index + 1, total));

    if !previous.is_empty() {
        s.push_str("\nSteps already written, so you do not repeat them:\n");
        for (i, t) in previous.iter().enumerate() {
            s.push_str(&format!("{}. {}\n", i + 1, t));
        }
    }
    if !planned_title.is_empty() {
        s.push_str(&format!(
            "\nA working title for this step is \"{planned_title}\". \
Keep it if the screenshot supports it, otherwise write a better one.\n"
        ));
    }

    s.push_str(
        "\nLook at the screenshot and return:\n\
- title: the single action the reader takes here. Imperative, at most 60 characters, no trailing full stop.\n\
- body: one to three sentences telling the reader exactly what to do and where to do it. \
Name the on-screen labels you can read. If the step's result is visible and worth confirming, \
mention it in the final sentence.",
    );
    s
}

/// JSON schema for the outline pass, sent as `responseSchema` so we get
/// parseable output instead of hoping the model formats it correctly.
pub fn outline_schema() -> serde_json::Value {
    serde_json::json!({
        "type": "object",
        "properties": {
            "title": { "type": "string" },
            "summary": { "type": "string" },
            "prerequisites": { "type": "array", "items": { "type": "string" } },
            "steps": { "type": "array", "items": { "type": "string" } }
        },
        "required": ["title", "summary", "steps"],
        "propertyOrdering": ["title", "summary", "prerequisites", "steps"]
    })
}

pub fn step_schema() -> serde_json::Value {
    serde_json::json!({
        "type": "object",
        "properties": {
            "title": { "type": "string" },
            "body": { "type": "string" }
        },
        "required": ["title", "body"],
        "propertyOrdering": ["title", "body"]
    })
}
