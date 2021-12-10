use chrono::{Datelike, Local, NaiveDate};
use druid::widget::{Flex, Label};
use druid::{AppLauncher, Color, Data, Lens, Widget, WidgetExt, WindowDesc};
use druid_calendar_grid::{CalendarGrid, YearAndMonth};

/// An example that shows the current month in a simple calendar view

#[derive(Clone, Copy, Data, Lens)]
struct AppData {
    #[data(same_fn = "PartialEq::eq")]
    date: YearAndMonth,
}

fn build_ui() -> impl Widget<AppData> {
    let title = Label::dynamic(|data: &AppData, _| data.date.first().format("%B – %Y").to_string())
        .center()
        .padding(16.0);

    let calendar = CalendarGrid::new(
        |data: &AppData, _| data.date,
        |date, _: &AppData, _| Box::new(build_day(date)),
    )
    .with_spacing(8.0)
    .padding((8.0, 0.0, 8.0, 8.0));

    Flex::column()
        .with_child(title)
        .with_flex_child(calendar, 1.0)
}

fn build_day(date: &NaiveDate) -> impl Widget<AppData> {
    let fmt = if date.day() == 1 { "%-d %b" } else { "%-d" };

    Label::new(date.format(fmt).to_string())
        .center()
        .border(Color::GRAY, 1.0)
        .rounded(4.0)
}

fn main() {
    let window = WindowDesc::new(build_ui);

    let data = AppData {
        date: Local::now().date().naive_local().into(),
    };

    // Launch
    AppLauncher::with_window(window)
        .launch(data)
        .expect("Failed to launch");
}
