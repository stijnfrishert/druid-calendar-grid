use chrono::{Datelike, NaiveDate};
use druid::{
    BoxConstraints, Data, Env, Event, EventCtx, KeyOrValue, LayoutCtx, LifeCycle, LifeCycleCtx,
    PaintCtx, Point, Size, UpdateCtx, Widget, WidgetPod,
};

const DAY_COUNT: usize = 5 * 7;

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
pub struct YearAndMonth {
    pub year: i32,
    pub month: u32, // January = 1
}

impl YearAndMonth {
    pub fn first(&self) -> NaiveDate {
        NaiveDate::from_ymd(self.year, self.month, 1)
    }

    pub fn pred(&self) -> Self {
        let mut year_and_month = *self;

        if year_and_month.month == 1 {
            year_and_month.year -= 1;
            year_and_month.month = 12;
        } else {
            year_and_month.month -= 1;
        }

        year_and_month
    }

    pub fn succ(&self) -> Self {
        let mut year_and_month = *self;

        if year_and_month.month == 12 {
            year_and_month.year += 1;
            year_and_month.month = 1;
        } else {
            year_and_month.month += 1;
        }

        year_and_month
    }
}

impl<T: Datelike> From<T> for YearAndMonth {
    fn from(datelike: T) -> Self {
        Self {
            year: datelike.year(),
            month: datelike.month(),
        }
    }
}

type DatePicker<T> = dyn FnMut(&T, &Env) -> YearAndMonth;
type DayBuilder<T> = dyn FnMut(&NaiveDate, &T, &Env) -> Box<dyn Widget<T>>;

type Child<T> = WidgetPod<T, Box<dyn Widget<T>>>;

pub struct CalendarGrid<T> {
    date_picker: Box<DatePicker<T>>,
    child_builder: Box<DayBuilder<T>>,
    children: Vec<Child<T>>,
    spacing: KeyOrValue<f64>,
}

impl<T> CalendarGrid<T> {
    pub fn new(
        year_and_month_picker: impl FnMut(&T, &Env) -> YearAndMonth + 'static,
        child_builder: impl FnMut(&NaiveDate, &T, &Env) -> Box<dyn Widget<T>> + 'static,
    ) -> Self {
        Self {
            date_picker: Box::new(year_and_month_picker),
            child_builder: Box::new(child_builder),
            children: Vec::new(),
            spacing: KeyOrValue::Concrete(0.),
        }
    }

    pub fn with_spacing<S: Into<KeyOrValue<f64>>>(mut self, spacing: S) -> Self {
        self.spacing = spacing.into();
        self
    }

    fn create_children(&mut self, year_and_month: &YearAndMonth, data: &T, env: &Env) {
        // Create a date on the 1st of the month
        let mut date = year_and_month.first();

        // Move back to Monday on or before the first of the month
        for _ in 0..date.weekday().num_days_from_monday() {
            date = date.pred();
        }

        // Create a child for each successive day
        self.children.clear();
        for _ in 0..DAY_COUNT {
            self.children
                .push(WidgetPod::new((self.child_builder)(&date, data, env)));
            date = date.succ();
        }
    }
}

impl<T: Data> Widget<T> for CalendarGrid<T> {
    fn event(&mut self, ctx: &mut EventCtx, event: &Event, data: &mut T, env: &Env) {
        for child in self.children.iter_mut() {
            child.event(ctx, event, data, env);
        }
    }

    fn lifecycle(&mut self, ctx: &mut LifeCycleCtx, event: &LifeCycle, data: &T, env: &Env) {
        if let LifeCycle::WidgetAdded = event {
            let year_and_month = (self.date_picker)(data, env);
            self.create_children(&year_and_month, data, env);
        }

        for child in self.children.iter_mut() {
            child.lifecycle(ctx, event, data, env);
        }
    }

    fn update(&mut self, ctx: &mut UpdateCtx, old_data: &T, data: &T, env: &Env) {
        // Check if the date changed
        let old_year_and_month = (self.date_picker)(old_data, env);
        let year_and_month = (self.date_picker)(data, env);

        if old_year_and_month != year_and_month {
            self.create_children(&year_and_month, data, env);
            ctx.children_changed();
        } else {
            for child in self.children.iter_mut() {
                child.update(ctx, data, env);
            }
        }
    }

    fn layout(&mut self, ctx: &mut LayoutCtx, bc: &BoxConstraints, data: &T, env: &Env) -> Size {
        assert!(bc.is_width_bounded() && bc.is_height_bounded());

        // Compute some spacing vars
        let spacing = self.spacing.resolve(env);
        let reduction = Size::new(spacing * 6.0, spacing * 4.0);

        // Deduce the BoxConstraints for each day
        let max = Size::new(
            (bc.max().width - reduction.width) / 7.0,
            (bc.max().height - reduction.height) / 5.0,
        );

        let day_bc = BoxConstraints::new(
            Size::new(
                (bc.min().width - reduction.width).max(0.0) / 7.0,
                (bc.min().height - reduction.height).max(0.0) / 5.0,
            ),
            max,
        );

        // Set the origin for each child
        for week in 0..5 {
            for day in 0..7 {
                let child = &mut self.children[week * 7 + day];
                child.layout(ctx, &day_bc, data, env);
                child.set_origin(
                    ctx,
                    data,
                    env,
                    Point::new(
                        (max.width + spacing) * day as f64,
                        (max.height + spacing) * week as f64,
                    ),
                );
            }
        }

        bc.max()
    }

    fn paint(&mut self, ctx: &mut PaintCtx, data: &T, env: &Env) {
        for child in self.children.iter_mut() {
            child.paint(ctx, data, env);
        }
    }
}
