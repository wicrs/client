use cursive::align::HAlign;
use cursive::event::Key;
use cursive::menu::MenuTree;
use cursive::theme::Theme;
use cursive::traits::Boxable;
use cursive::view::SizeConstraint;
use cursive::views::Button;
use cursive::views::Dialog;
use cursive::views::EditView;
use cursive::views::FixedLayout;
use cursive::views::LinearLayout;
use cursive::views::ListView;
use cursive::views::Menubar;
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

#[derive(Debug)]
struct State {
    user_id: Uuid,
    selected_hub: Option<Uuid>,
    selected_channel: Option<Uuid>,
    hubs: HashMap<Uuid, Hub>,
}

#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
    let mut cursive_runner = cursive::default();
    let user_id = Uuid::new_v4();
    let hub_id = Uuid::new_v4();
    let mut hub = Hub::new("test0".to_string(), hub_id, user_id);
    hub.description = "A hub for testing WICRS client.".to_string();
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
    for i in 1..10 {
        let id = Uuid::new_v4();
        hubs.insert(id, Hub::new(format!("test{}", i), id, user_id));
    }
    for hub in hubs.values_mut() {
        for i in 0..10 {
            hub.new_channel(
                &user_id,
                format!("test{}", i),
                "A channel for testing.".to_string(),
            )
            .await
            .unwrap();
        }
    }
    cursive_runner.set_user_data(State {
        user_id,
        selected_hub: Some(hub_id),
        selected_channel: Some(channel_id),
        hubs,
    });
    start_render(&mut cursive_runner).await;
    Ok(())
}

async fn start_render(c: &mut CursiveRunnable) {
    let theme = Theme {
        shadow: false,
        ..Default::default()
    };
    menubar(c.menubar());
    c.add_global_callback(Key::Esc, |s| s.select_menubar());
    c.set_theme(theme);
    render(c).await;
    c.run();
}

async fn render(c: &mut Cursive) {
    c.pop_layer();
    let state = c.user_data::<State>().unwrap();
    let hub = hub_area(state).await;
    let channel = channel_area(state).await;
    let message = message_area(state).await;
    let user = Panel::new(TextView::new(format!(
        "Eventually user info will be here...\nID: {}",
        state.user_id.to_string()
    )))
    .title("Users")
    .title_position(HAlign::Left);
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
}

fn srender(c: &mut Cursive) {
    block_on(render(c));
}

async fn channel_area(state: &mut State) -> LinearLayout {
    let mut layout = LinearLayout::vertical();
    if let Some(hub_id) = state.selected_hub {
        let hub = state.hubs.get(&hub_id).unwrap();
        let mut channel_list = ListView::default();
        for (id, channel) in &hub.channels {
            let id = *id;
            let item = Button::new_raw(&channel.name, move |c| {
                c.user_data::<State>().unwrap().selected_channel = Some(id);
                srender(c);
            });
            channel_list.add_child("", item)
        }
        layout.add_child(
            Panel::new(channel_list.full_height())
                .title("Channels")
                .title_position(HAlign::Left),
        );
    } else {
        layout.add_child(
            Panel::new(TextView::new("").full_height())
                .title("Channels")
                .title_position(HAlign::Left),
        );
    }
    layout
}

async fn message_area(state: &mut State) -> LinearLayout {
    let mut layout = LinearLayout::vertical();
    if let Some(hub_id) = state.selected_hub {
        let hub = state.hubs.get(&hub_id).unwrap();
        let hub_text = format!("Name: {}\nDescription: {}", hub.name, hub.description);
        layout.add_child(
            Panel::new(TextView::new(hub_text))
                .title("Current Hub")
                .title_position(HAlign::Left),
        );
        if let Some(channel_id) = state.selected_channel {
            let channel = hub.channels.get(&channel_id).unwrap();
            let channel_text = format!(
                "Name: {}\nDescription: {}",
                channel.name, channel.description
            );
            let mut message_list = ListView::default();

            for message in channel.get_last_messages(100).await {
                message_list.add_child(
                    "",
                    TextView::new(format!(
                        "{} [{}]: {}",
                        message.created.format("%H:%M:%S").to_string(),
                        message.sender.to_string(),
                        message.content
                    )),
                )
            }
            layout.add_child(
                Panel::new(TextView::new(channel_text))
                    .title("Current Channel")
                    .title_position(HAlign::Left),
            );
            let mut scroll = ScrollView::new(message_list);
            scroll.scroll_to_bottom();
            layout.add_child(
                Panel::new(scroll.full_height())
                    .title("Messages")
                    .title_position(HAlign::Left),
            );
            layout.add_child(
                Panel::new(EditView::new().on_submit(|c, text| {
                    let state = c.user_data::<State>().unwrap();
                    let hub = state.hubs.get(&state.selected_hub.unwrap()).unwrap();
                    let channel = hub.channels.get(&state.selected_channel.unwrap()).unwrap();
                    block_on(channel.add_message(Message::new(
                        state.user_id,
                        text.to_string(),
                        state.selected_hub.unwrap(),
                        state.selected_channel.unwrap(),
                    )))
                    .unwrap();
                    srender(c)
                }))
                .title("New Message")
                .title_position(HAlign::Left),
            );
        } else {
            layout.add_child(Panel::new(
                TextView::new("Please select a channel...").full_height(),
            ));
        }
    } else {
        layout.add_child(Panel::new(
            TextView::new("Please select a hub...").full_height(),
        ));
    }
    layout
}

async fn hub_area(state: &mut State) -> Panel<ListView> {
    let mut hub_list = ListView::default();
    for (id, hub) in &state.hubs {
        let id = *id;
        let item = Button::new_raw(&hub.name, move |c| {
            let user_data = c.user_data::<State>().unwrap();
            user_data.selected_hub = Some(id);
            user_data.selected_channel = None;
            srender(c);
        });
        hub_list.add_child("", item)
    }
    Panel::new(hub_list)
        .title("Hubs")
        .title_position(HAlign::Left)
}

fn menubar(menubar: &mut Menubar) {
    menubar.add_leaf("Quit", |s| s.quit()).add_subtree(
        "Hubs",
        MenuTree::default().leaf("Join...", |s| {
            s.add_layer(
                Dialog::default()
                    .title("Join Hub")
                    .dismiss_button("X")
                    .content(
                        ListView::new()
                            .child("", TextView::new("Enter UUID of hub to join:"))
                            .child(
                                "",
                                EditView::default()
                                    .on_submit(|s, _text| {
                                        s.pop_layer();
                                    })
                                    .resized(SizeConstraint::AtLeast(36), SizeConstraint::Free),
                            ),
                    ),
            )
        }),
    );
}
