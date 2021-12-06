use cursive::align::HAlign;
use cursive::menu::MenuTree;
use cursive::theme::Theme;
use cursive::traits::Boxable;
use cursive::view::SizeConstraint;
use cursive::views::Dialog;
use cursive::views::EditView;
use cursive::views::LinearLayout;
use cursive::views::ListView;
use cursive::views::Menubar;
use cursive::views::OnLayoutView;
use cursive::views::Panel;
use cursive::views::TextArea;
use cursive::views::TextView;
use cursive::CursiveRunnable;
use cursive::View;
use std::collections::HashMap;
use uuid::Uuid;
use wicrs_api::wicrs_server::channel::Message;
use wicrs_api::wicrs_server::prelude::Hub;

#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
    let siv = cursive::default();
    let user_id = Uuid::new_v4();
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
    render(siv);
    Ok(())
}

fn render(mut c: CursiveRunnable) {
    let mut theme = Theme::default();
    theme.shadow = false;
    menubar(c.menubar());
    c.set_autohide_menu(false);
    c.set_theme(theme);
    let layout = OnLayoutView::new(LinearLayout::horizontal(), |v, s| {
        let width = dbg!(s).x;
        let mut remaining_width = width;
        let hub_width = (s.x as f32 * 15f32 / 100f32) as usize;
        remaining_width -= hub_width;
        v.add_child(hub_area().fixed_width(hub_width));
        v.add_child(channel_area().fixed_width(remaining_width));
        v.layout(s)
    });
    c.add_fullscreen_layer(layout);
    c.run();
}

fn channel_area() -> LinearLayout {
    LinearLayout::vertical()
        .child(
            Panel::new(TextView::new(
                "Name: {placeholder for name}\nDescription: {placeholder for description}",
            ))
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

fn hub_area() -> LinearLayout {
    LinearLayout::vertical()
        .child(
            Panel::new(TextView::new(
                "Name: {placeholder for name}\nDescription: {placeholder for description}",
            ))
            .title("Current Hub")
            .title_position(HAlign::Left),
        )
        .child(
            Panel::new(ListView::default().full_height())
                .title("Hubs")
                .title_position(HAlign::Left),
        )
}

fn menubar(menubar: &mut Menubar) {
    menubar.add_leaf("X", |s| s.quit()).add_subtree(
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
