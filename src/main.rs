use std::collections::HashMap;
use std::io::{self, Stdout};
use tui::backend::CrosstermBackend;
use tui::layout::{Constraint, Direction, Layout};
use tui::text::{Spans, Text};
use tui::widgets::{Block, Borders, Paragraph};
use tui::Terminal;
use uuid::Uuid;
use wicrs_api::wicrs_server::channel::Message;
use wicrs_api::wicrs_server::prelude::Hub;

#[tokio::main]
async fn main() -> Result<(), io::Error> {
    let user_id = Uuid::new_v4();
    let stdout = io::stdout();
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    terminal.clear()?;
    let hub_id = Uuid::new_v4();
    let mut hub = Hub::new("test0".to_string(), hub_id, user_id);
    let channel_id = hub
        .new_channel(&user_id, "chat".to_string(), "A place to chat.".to_string())
        .await
        .unwrap();
    let channel = hub.channels.get(&channel_id).unwrap();
    for i in 0..10usize {
        channel
            .add_message(Message::new(
                user_id,
                format!("Test message {:02}.", i),
                hub_id,
                channel_id,
            ))
            .await
            .unwrap();
    }
    let mut hubs = HashMap::new();
    hubs.insert(hub_id, hub);
    render(&mut terminal, &user_id, &channel_id, &hub_id, &hubs).await?;
    Ok(())
}

async fn render(
    terminal: &mut Terminal<CrosstermBackend<Stdout>>,
    user_id: &Uuid,
    current_channel: &Uuid,
    hub: &Uuid,
    hubs: &HashMap<Uuid, Hub>,
) -> io::Result<()> {
    let hchunks = Layout::default()
        .direction(Direction::Horizontal)
        .margin(0)
        .constraints([
            Constraint::Percentage(10),
            Constraint::Percentage(10),
            Constraint::Percentage(70),
            Constraint::Percentage(10),
        ])
        .split(terminal.size()?);
    let vchunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(0)
        .constraints([Constraint::Percentage(20), Constraint::Percentage(80)]);
    let hub_chunks = vchunks.split(hchunks[0]);
    let channel_chunks = vchunks.split(hchunks[1]);
    let message_chunks = Layout::default()
        .margin(0)
        .constraints([Constraint::Percentage(90), Constraint::Percentage(10)])
        .split(hchunks[2]);
    let user_chunks = vchunks.split(hchunks[3]);

    let hub = hubs.get(hub).expect("Cannot find current hub.");
    let channel = hub
        .channels
        .get(current_channel)
        .expect("Current hub does not contain current channel.");

    let current_hub = create_paragraph(hub.name.to_string(), hub.description.to_string());
    let hub_list = create_paragraph(
        "Hubs",
        iter_to_text(hubs.iter().map(|(_, h)| h.name.to_string())),
    );
    let current_channel =
        create_paragraph(channel.name.to_string(), channel.description.to_string());
    let channel_list = create_paragraph(
        "Channels",
        iter_to_text(hub.channels.iter().map(|(_, c)| c.name.to_string())),
    );
    let message_list = create_paragraph(
        "Messages",
        iter_to_text(dbg!(channel.get_last_messages(100).await).iter().map(|m| {
            format!(
                "{} [{}] {}",
                m.created.format("%H:%M:%S").to_string(),
                m.sender.to_string(),
                m.content
            )
        })),
    );
    let current_user = create_paragraph("You", user_id.to_string());
    let user_list = create_paragraph(
        "Users",
        iter_to_text(hub.members.iter().map(|(i, _)| i.to_string())),
    );
    terminal.draw(|f| {
        f.render_widget(current_hub, hub_chunks[0]);
        f.render_widget(hub_list, hub_chunks[1]);
        f.render_widget(current_channel, channel_chunks[0]);
        f.render_widget(channel_list, channel_chunks[1]);
        f.render_widget(message_list, message_chunks[0]);
        f.render_widget(current_user, user_chunks[0]);
        f.render_widget(user_list, user_chunks[1]);
    })?;
    Ok(())
}

fn iter_to_text<'a, I, S>(iter: I) -> Text<'a>
where
    S: std::fmt::Display,
    I: IntoIterator<Item = S>,
{
    let mut text = Text::default();
    for item in iter {
        text.extend(Text::from(format!("{}\n", item)));
    }
    text
}

fn create_paragraph<'a, T, S>(title: S, text: T) -> Paragraph<'a>
where
    T: Into<Text<'a>>,
    S: Into<Spans<'a>>,
{
    Paragraph::new(text).block(Block::default().borders(Borders::ALL).title(title))
}
