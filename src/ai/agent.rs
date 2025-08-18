use crate::event::EventCategory;
use crate::model::Message;
use agentai::Agent;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

pub fn init_categorize_event_agent() -> Agent {
    Agent::new(
        r#"You are an agent that categorizes events based on the provided description.
        You will receive a description of an event and you need to categorize it into one of the following categories:
        - BirthEarlyYears
        - FamilyRelationships
        - EducationLearning
        - CareerWork
        - ResidenceRelocation
        - HealthWellbeing
        - FinanceWealth
        - CityCommunity
        - CultureSpirituality
        - TravelAdventure
        - HobbySportLeisure
        - DigitalTech
        - EnvironmentNature
        - LaterLife
        - PassingCommemoration
    "#,
    )
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
struct CategoryAnswer {
    #[serde(rename = "_thinking")]
    thinking: String,
    category: EventCategory,
}

pub async fn categorize_with_agent(
    agent: &mut Agent,
    event_name: &str,
    description: &str,
    date: &str,
) -> Result<EventCategory, String> {
    let input = format!(
        "Event Name: {}\nDescription: {}\nDate: {}",
        event_name, description, date
    );

    let answer: CategoryAnswer = agent
        .run("gpt-4o", &input, None)
        .await
        .map_err(|e| e.to_string())?;

    Ok(answer.category)
}

pub fn init_conversation_agent() -> Agent {
    Agent::new(
        r#"You are an helpful agent that ask questions to the user 
about the life events they are describing.

You'll receive the name of the event, a description.

the date and the passed conversation between you and the user.


Your main goal is to help people recover their memories in order to elaborate the description of the event.

Focus on:
- The people that were there
- The place where it happened
- The emotions felt
"#,
    )
}

fn build_input_message_payload(
    event_name: &str,
    description: &str,
    date: &str,
    conversation: Vec<Message>,
) -> String {
    let mut input = String::new();
    input.push_str(&format!("Event Name: {}\n", event_name));
    input.push_str(&format!("Description: {}\n", description));
    input.push_str(&format!("Date: {}\n", date));
    input.push_str("Conversation:\n");

    for msg in conversation {
        input.push_str(&format!("{}: {}\n", msg.source, msg.content));
    }

    input
}

pub async fn ask_question_agent(
    agent: &mut Agent,
    event_name: &str,
    description: &str,
    date: &str,
    conversation: Vec<Message>,
) -> Result<String, String> {
    let input = build_input_message_payload(event_name, description, date, conversation);

    let answer = agent
        .run("gpt-4o", &input, None)
        .await
        .map_err(|e| e.to_string())?;

    Ok(answer)
}
