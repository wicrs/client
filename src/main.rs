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
use cursive::views::TextArea;
use cursive::views::TextView;
use cursive::CursiveRunnable;
use cursive::Rect;
use cursive::View;
use cursive::XY;
use std::collections::HashMap;
use uuid::Uuid;
use wicrs_api::wicrs_server::channel::Message;
use wicrs_api::wicrs_server::prelude::Hub;

#[derive(Debug)]
struct State {
    user_id: Uuid,
    selected_hub: Uuid,
    selected_channel: Uuid,
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
    for (_, hub) in &mut hubs {
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
        selected_hub: hub_id,
        selected_channel: channel_id,
        hubs,
    });
    render(&mut cursive_runner);
    Ok(())
}

fn render(c: &mut CursiveRunnable) {
    let mut theme = Theme::default();
    theme.shadow = false;
    menubar(c.menubar());
    c.add_global_callback(Key::Esc, |s| s.select_menubar());
    c.set_theme(theme);
    let hub = hub_area(c);
    let channel = channel_area(c);
    let message = message_area(c);
    let user = Panel::new(TextView::new(format!(
        "Eventually user info will be here...\nID: {}",
        c.user_data::<State>().unwrap().user_id.to_string()
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
            .x;
        let mut channel_width = s.x * 15 / 100;
        channel_width = fixed_layout
            .get_child_mut(1)
            .unwrap()
            .required_size(XY::new(channel_width, s.y))
            .x;
        let mut user_width = s.x * 15 / 100;
        user_width = fixed_layout
            .get_child_mut(3)
            .unwrap()
            .required_size(XY::new(user_width, s.y))
            .x;
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
    c.run();
}

fn channel_area(c: &mut CursiveRunnable) -> LinearLayout {
    dbg!(c.user_data::<State>().unwrap().selected_channel);
    let state = c.user_data::<State>().unwrap();
    let hub = state.hubs.get(&state.selected_hub).unwrap();
    let mut channel_list = ListView::default();
    for (id, channel) in &hub.channels {
        let id = *id;
        let item = Button::new_raw(
            if id == state.selected_channel {
                format!("<{}>", channel.name)
            } else {
                channel.name.clone()
            },
            move |c| {
                c.user_data::<State>().unwrap().selected_channel = id;
                println!("selected channel {}", id.to_string())
            },
        );
        channel_list.add_child("", item)
    }
    LinearLayout::vertical().child(
        Panel::new(channel_list.full_height())
            .title("Messages")
            .title_position(HAlign::Left),
    )
}

fn message_area(c: &mut CursiveRunnable) -> LinearLayout {
    let state = c.user_data::<State>().unwrap();
    let hub = state.hubs.get(&state.selected_hub).unwrap();
    let hub_text = format!("Name: {}\nDescription: {}", hub.name, hub.description);
    let channel = hub.channels.get(&state.selected_channel).unwrap();
    let channel_text = format!(
        "Name: {}\nDescription: {}",
        channel.name, channel.description
    );
    LinearLayout::vertical()
        .child(
            Panel::new(TextView::new(hub_text))
                .title("Current Hub")
                .title_position(HAlign::Left),
        )
        .child(
            Panel::new(TextView::new(channel_text))
                .title("Current Channel")
                .title_position(HAlign::Left),
        )
        .child(
            Panel::new(ListView::default().full_height())
                .title("Messages")
                .title_position(HAlign::Left),
        )
        .child(
            Panel::new(TextArea::default().content("Write new message here..."))
                .title("New Message")
                .title_position(HAlign::Left),
        )
}

fn hub_area(c: &mut CursiveRunnable) -> Panel<ListView> {
    let state = c.user_data::<State>().unwrap();
    let mut hub_list = ListView::default();
    for (id, hub) in &state.hubs {
        let id = *id;
        let item = Button::new_raw(
            if id == state.selected_hub {
                format!("<{}>", hub.name)
            } else {
                hub.name.clone()
            },
            move |c| {
                c.user_data::<State>().unwrap().selected_hub = id;
                println!("selected hub {}", id.to_string())
            },
        );
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
