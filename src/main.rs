use cursive::align::HAlign;
use cursive::menu::MenuTree;
use cursive::theme::Theme;
use cursive::traits::Boxable;
use cursive::traits::Nameable;
use cursive::view::SizeConstraint;
use cursive::views::Button;
use cursive::views::Dialog;
use cursive::views::EditView;
use cursive::views::FixedLayout;
use cursive::views::LinearLayout;
use cursive::views::ListView;
use cursive::views::Menubar;
use cursive::views::NamedView;
use cursive::views::OnLayoutView;
use cursive::views::Panel;
use cursive::views::ResizedView;
use cursive::views::ScrollView;
use cursive::views::TextView;
use cursive::Cursive;
use cursive::CursiveRunnable;
use cursive::Rect;
use cursive::View;
use cursive::XY;
use futures::executor::block_on;
use std::collections::HashMap;
use uuid::Uuid;
use wicrs_api::wicrs_server::channel::Message;
use wicrs_api::wicrs_server::prelude::Hub;
use wicrs_api::Result;

#[derive(Debug)]
struct State {
    user_id: Uuid,
    selected_hub: Option<Uuid>,
    selected_channel: Option<Uuid>,
    hubs: HashMap<Uuid, Hub>,
}

#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
    let mut cursive_runner = cursive::crossterm();
    let user_id = Uuid::from_u128(128);
    std::fs::create_dir_all("data/hubs/info").unwrap();
    let mut hubs = HashMap::new();
    for file in std::fs::read_dir("data/hubs/info").unwrap() {
        let id = Uuid::parse_str(&file.unwrap().file_name().to_string_lossy()).unwrap();
        let mut hub = Hub::load(id).await.unwrap();
        if hub.user_join(user_id).is_ok() {
            let _ = hub.save().await;
        }
        hubs.insert(id, hub);
    }
    if hubs.is_empty() {
        for h in 0..9u128 {
            let mut hub = Hub::new(format!("hub-{}", h), Uuid::from_u128(h), user_id);
            hub.description = format!("A hub for testing (#{}).", h);
            for c in 0..9u128 {
                let _ = hub
                    .new_channel(
                        &user_id,
                        format!("channel-{}", c),
                        format!("A channel for testing (#{}).", c),
                    )
                    .await;
            }
            let _ = hub.save().await;
            hubs.insert(hub.id, hub);
        }
    }
    cursive_runner.set_user_data(State {
        user_id,
        selected_hub: None,
        selected_channel: None,
        hubs,
    });
    start_render(&mut cursive_runner);
    Ok(())
}

fn start_render(c: &mut CursiveRunnable) {
    let theme = Theme {
        shadow: false,
        ..Default::default()
    };
    c.set_autohide_menu(false);
    c.set_theme(theme);
    menubar(c.menubar());
    render(c);
    c.run();
}

fn render(c: &mut Cursive) {
    c.pop_layer();
    let state = get_state(c);
    let hub = hub_area();
    let channel = channel_area();
    let message = message_area();
    let user = Panel::new(TextView::new(format!(
        "Eventually user info will be here...\nID: {}",
        state.user_id.to_string()
    )))
    .title("Users")
    .title_position(HAlign::Left);
    let mut hubs = Vec::new();
    for (id, hub) in &state.hubs {
        hubs.push((*id, hub.name.clone()));
    }
    let resized_view = ResizedView::new(
        SizeConstraint::Full,
        SizeConstraint::Full,
        FixedLayout::new()
            .child(Rect::from_size((0, 0), (1, 1)), hub)
            .child(Rect::from_size((0, 0), (1, 1)), channel)
            .child(Rect::from_size((0, 0), (1, 1)), message)
            .child(Rect::from_size((0, 0), (1, 1)), user),
    );
    c.add_fullscreen_layer(OnLayoutView::new(resized_view, |v, s| {
        let fixed_layout = v.get_inner_mut();
        let mut hub_width = s.x * 15 / 100;
        hub_width = fixed_layout
            .get_child_mut(0)
            .unwrap()
            .required_size(XY::new(hub_width, s.y))
            .x
            + 1;
        let mut channel_width = s.x * 15 / 100;
        channel_width = fixed_layout
            .get_child_mut(1)
            .unwrap()
            .required_size(XY::new(channel_width, s.y))
            .x
            + 1;
        let mut user_width = s.x * 15 / 100;
        user_width = fixed_layout
            .get_child_mut(3)
            .unwrap()
            .required_size(XY::new(user_width, s.y))
            .x
            + 1;
        let message_width = s.x - hub_width - channel_width - user_width;
        let hub_pos = Rect::from_size((0, 0), (hub_width, s.y));
        let channel_pos = Rect::from_size((hub_width + 1, 0), (channel_width, s.y));
        let message_pos = Rect::from_size((hub_width + channel_width + 2, 0), (message_width, s.y));
        let user_pos = Rect::from_size(
            (hub_width + channel_width + message_width + 3, 0),
            (user_width, s.y),
        );
        fixed_layout.set_child_position(0, hub_pos);
        fixed_layout.set_child_position(1, channel_pos);
        fixed_layout.set_child_position(2, message_pos);
        fixed_layout.set_child_position(3, user_pos);
        v.layout(s)
    }));
    for (id, name) in hubs {
        add_hub_to_list(c, id, &name);
    }
}

fn message_area() -> LinearLayout {
    LinearLayout::vertical()
        .child(
            Panel::new(TextView::new("No hub selected...").with_name("hub_info"))
                .title("Hub Info")
                .title_position(HAlign::Left),
        )
        .child(
            Panel::new(TextView::new("No channel selected...").with_name("channel_info"))
                .title("Channel Info")
                .title_position(HAlign::Left),
        )
        .child(
            Panel::new(
                ScrollView::new(ListView::default().with_name("message_list")).full_height(),
            )
            .title("Messages")
            .title_position(HAlign::Left),
        )
        .child(
            Panel::new(
                EditView::new()
                    .on_submit(|c, text| {
                        let state = get_state(c);
                        if let (Some(hub_id), Some(channel_id)) =
                            (state.selected_hub, state.selected_channel)
                        {
                            if let Some(hub) = state.hubs.get(&hub_id) {
                                let send = block_on(hub.send_message(
                                    state.user_id,
                                    channel_id,
                                    text.to_string(),
                                ));
                                if let Ok(m) = send {
                                    new_message(c, m);
                                    c.call_on_name("message_box", |v: &mut EditView| {
                                        v.set_content("");
                                    });
                                    c.focus_name("message_box").unwrap();
                                } else {
                                    error(
                                        c,
                                        &format!(
                                            "Failed to send message: {}",
                                            send.unwrap_err().to_string()
                                        ),
                                    );
                                }
                            } else {
                                error(c, "Could not find selected hub.");
                            }
                        } else {
                            error(
                                c,
                                "You must select a hub and a channel before sending messages.",
                            );
                        }
                    })
                    .with_name("message_box"),
            )
            .title("Send Message")
            .title_position(HAlign::Left),
        )
}

fn new_message(c: &mut Cursive, message: Message) {
    let state = get_state(c);
    if let (Some(hub_id), Some(channel_id)) = (state.selected_hub, state.selected_channel) {
        if message.hub_id == hub_id && message.channel_id == channel_id {
            c.call_on_name("message_list", |v: &mut ListView| {
                v.add_child("", TextView::new(message_string(message)))
            });
        }
    }
}

fn message_string(message: Message) -> String {
    format!(
        "{} [{}]: {}",
        message
            .created
            .with_timezone(&chrono::Local)
            .format("%H:%M:%S")
            .to_string(),
        message.sender.to_string(),
        message.content
    )
}

fn channel_area() -> Panel<ResizedView<NamedView<ListView>>> {
    Panel::new(ListView::default().with_name("channel_list").full_height())
        .title("Channels")
        .title_position(HAlign::Left)
}

fn hub_area() -> Panel<NamedView<ListView>> {
    Panel::new(ListView::default().with_name("hub_list"))
        .title("Hubs")
        .title_position(HAlign::Left)
}

fn add_hub_to_list(c: &mut Cursive, id: Uuid, name: &str) {
    let item = Button::new_raw(name, move |c| {
        select_hub(c, id);
    });
    c.call_on_name("hub_list", |v: &mut ListView| v.add_child("", item));
}

fn add_channel_to_list(c: &mut Cursive, id: Uuid, name: &str) {
    let item = Button::new_raw(name, move |c| {
        select_channel(c, id);
    });
    c.call_on_name("channel_list", |v: &mut ListView| v.add_child("", item));
}

fn get_state(c: &mut Cursive) -> &mut State {
    c.user_data::<State>()
        .expect("unable to get application state")
}

fn select_hub(c: &mut Cursive, id: Uuid) {
    let state = get_state(c);
    state.selected_hub = Some(id);
    state.selected_channel = None;
    let name;
    let description;
    let mut channels = Vec::new();
    if let Some(hub) = state.hubs.get(&id) {
        name = hub.name.clone();
        description = hub.description.clone();
        for (id, channel) in &hub.channels {
            channels.push((*id, channel.name.clone()));
        }
    } else {
        error(c, "Hub does not exist locally.");
        return;
    }
    c.call_on_name("hub_info", |v: &mut TextView| {
        v.set_content(format!("Name: {}\nDescription: {}", name, description));
    });
    c.call_on_name("channel_info", |v: &mut TextView| {
        v.set_content("No channel selected...");
    });
    c.call_on_name("channel_list", |v: &mut ListView| v.clear());
    c.call_on_name("message_list", |v: &mut ListView| v.clear());
    for (id, name) in channels {
        add_channel_to_list(c, id, &name);
    }
}

fn select_channel(c: &mut Cursive, id: Uuid) {
    let mut messages = Vec::new();
    let name;
    let description;
    let state = get_state(c);
    if let Some(hub_id) = state.selected_hub {
        if let Some(hub) = state.hubs.get(&hub_id) {
            if let Some(channel) = hub.channels.get(&id) {
                name = channel.name.clone();
                description = channel.description.clone();
                for message in block_on(channel.get_last_messages(200)) {
                    messages.push(message_string(message));
                }
            } else {
                error(c, "Could not find channel.");
                return;
            }
        } else {
            error(c, "Could not find hub.");
            return;
        }
    } else {
        error(c, "No hub selected...");
        return;
    }
    get_state(c).selected_channel = Some(id);
    c.call_on_name("channel_info", |v: &mut TextView| {
        v.set_content(format!("Name: {}\nDescription: {}", name, description));
    });
    c.call_on_name("message_list", |v: &mut ListView| {
        v.clear();
        for message in messages {
            v.add_child("", TextView::new(message));
        }
    });
}

fn error(c: &mut Cursive, message: &str) {
    c.add_layer(
        Dialog::default()
            .title("Error")
            .dismiss_button("Ok")
            .content(TextView::new(message)),
    );
}

fn menubar(menubar: &mut Menubar) {
    menubar.add_subtree(
        "Hubs",
        MenuTree::default()
            .leaf("Join", |c| {
                c.add_layer(
                    Dialog::default()
                        .title("Join Hub")
                        .dismiss_button("Cancel")
                        .content(
                            ListView::new()
                                .child("", TextView::new("Enter UUID of hub to join:"))
                                .child(
                                    "",
                                    EditView::default()
                                        .on_submit(|c, _text| {
                                            c.pop_layer();
                                        })
                                        .resized(SizeConstraint::AtLeast(36), SizeConstraint::Free),
                                ),
                        ),
                )
            })
            .leaf("Create", |c| {
                c.add_layer(
                    Dialog::default()
                        .title("Create Hub")
                        .dismiss_button("Cancel")
                        .content(
                            ListView::new()
                                .child("", TextView::new("Enter name of new hub:"))
                                .child(
                                    "",
                                    EditView::default()
                                        .on_submit(|c, text| {
                                            let state = get_state(c);
                                            let hub_id = Uuid::new_v4();
                                            let hub =
                                                Hub::new(text.to_string(), hub_id, state.user_id);
                                            if let Err(e) = block_on(hub.save()) {
                                                c.pop_layer();
                                                error(
                                                    c,
                                                    &format!(
                                                        "Could not save the new hub: {}",
                                                        e.to_string()
                                                    ),
                                                );
                                            } else {
                                                let name = hub.name.clone();
                                                state.hubs.insert(hub_id, hub);
                                                add_hub_to_list(c, hub_id, &name);
                                                select_hub(c, hub_id);
                                                c.pop_layer();
                                            }
                                        })
                                        .resized(SizeConstraint::AtLeast(36), SizeConstraint::Free),
                                ),
                        ),
                )
            }),
    );
    menubar.add_subtree(
        "Channels",
        MenuTree::default().leaf("Create", |c| {
            c.add_layer(
                Dialog::default()
                    .title("Create Channel")
                    .dismiss_button("Cancel")
                    .content(
                        ListView::new()
                            .child("", TextView::new("Enter name of new channel:"))
                            .child(
                                "",
                                EditView::default()
                                    .on_submit(|c, text| {
                                        let state = get_state(c);
                                        if let Some(hub_id) = state.selected_hub {
                                            let hub = state.hubs.get_mut(&hub_id).unwrap();
                                            let channel = block_on(hub.new_channel(
                                                &state.user_id,
                                                text.to_string(),
                                                String::new(),
                                            ));
                                            if let Err(e) = block_on(hub.save()) {
                                                c.pop_layer();
                                                error(
                                                    c,
                                                    &format!(
                                                        "Could not save the hub: {}",
                                                        e.to_string()
                                                    ),
                                                );
                                            } else {
                                                let channel = channel.unwrap();
                                                add_channel_to_list(c, channel, text);
                                                c.pop_layer();
                                            }
                                        } else {
                                            error(
                                                c,
                                                "You must select a hub before creating a channel.",
                                            )
                                        }
                                    })
                                    .resized(SizeConstraint::AtLeast(36), SizeConstraint::Free),
                            ),
                    ),
            )
        }),
    );
}
