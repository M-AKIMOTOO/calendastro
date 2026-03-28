use eframe::egui::{self, Align, Color32, RichText, TextEdit, Vec2};
use std::time::Duration;
use time::{OffsetDateTime, UtcOffset};

fn main() -> eframe::Result<()> {
    let window_size = [970.0, 900.0];
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size(window_size)
            .with_min_inner_size(window_size)
            .with_max_inner_size(window_size)
            .with_resizable(false),
        ..Default::default()
    };

    eframe::run_native(
        "calendastro",
        options,
        Box::new(|_cc| Ok(Box::new(CalendarApp::default()))),
    )
}

struct CalendarApp {
    year: i32,
    month: u32,
    year_input: String,
    month_input: String,
    error_message: Option<String>,
}

impl Default for CalendarApp {
    fn default() -> Self {
        let (year, month, _) = today_ymd();
        Self {
            year,
            month,
            year_input: year.to_string(),
            month_input: month.to_string(),
            error_message: None,
        }
    }
}

impl eframe::App for CalendarApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        ctx.request_repaint_after(Duration::from_secs(1));

        egui::CentralPanel::default().show(ctx, |ui| {
            let (jst_text, utc_text) = current_clock_strings();
            let controls_font = 15.0;

            ui.add_space(8.0);
            ui.horizontal_wrapped(|ui| {
                ui.heading("calendastro");
                ui.label(format!("{} {}", month_name(self.month), self.year));
            });
            ui.add_space(4.0);
            ui.vertical(|ui| {
                ui.label(RichText::new(jst_text).size(54.0).strong());
                ui.label(RichText::new(utc_text).size(54.0).strong());
            });
            ui.add_space(14.0);

            ui.horizontal_wrapped(|ui| {
                if ui
                    .add_sized(
                        [95.0, 36.0],
                        egui::Button::new(RichText::new("<< Prev").size(controls_font)),
                    )
                    .clicked()
                {
                    self.shift_month(-1);
                }
                if ui
                    .add_sized(
                        [80.0, 36.0],
                        egui::Button::new(RichText::new("Today").size(controls_font)),
                    )
                    .clicked()
                {
                    let (year, month, _) = today_ymd();
                    self.set_month(year, month);
                }
                if ui
                    .add_sized(
                        [95.0, 36.0],
                        egui::Button::new(RichText::new("Next >>").size(controls_font)),
                    )
                    .clicked()
                {
                    self.shift_month(1);
                }

                ui.separator();
                ui.label(RichText::new("Year").size(controls_font));
                ui.add_sized(
                    [80.0, 36.0],
                    TextEdit::singleline(&mut self.year_input).font(egui::TextStyle::Heading),
                );
                ui.label(RichText::new("Month").size(controls_font));
                ui.add_sized(
                    [60.0, 36.0],
                    TextEdit::singleline(&mut self.month_input).font(egui::TextStyle::Heading),
                );
                if ui
                    .add_sized(
                        [60.0, 36.0],
                        egui::Button::new(RichText::new("Go").size(controls_font)),
                    )
                    .clicked()
                {
                    self.apply_inputs();
                }
            });

            if let Some(message) = &self.error_message {
                ui.add_space(4.0);
                ui.colored_label(Color32::from_rgb(185, 40, 40), message);
            }

            ui.add_space(10.0);
            self.draw_calendar(ui);
        });
    }
}

impl CalendarApp {
    fn set_month(&mut self, year: i32, month: u32) {
        self.year = year;
        self.month = month;
        self.year_input = year.to_string();
        self.month_input = month.to_string();
        self.error_message = None;
    }

    fn shift_month(&mut self, delta: i32) {
        let month_index = self.month as i32 - 1 + delta;
        let year = self.year + month_index.div_euclid(12);
        let month = month_index.rem_euclid(12) as u32 + 1;
        self.set_month(year, month);
    }

    fn apply_inputs(&mut self) {
        let year = self.year_input.trim().parse::<i32>();
        let month = self.month_input.trim().parse::<u32>();

        match (year, month) {
            (Ok(year), Ok(month @ 1..=12)) => self.set_month(year, month),
            (Ok(_), Ok(_)) => {
                self.error_message = Some("Month must be in the range 1..=12.".to_owned());
            }
            _ => {
                self.error_message = Some("Enter numeric values for year and month.".to_owned());
            }
        }
    }

    fn draw_calendar(&self, ui: &mut egui::Ui) {
        let first_weekday = weekday_index(self.year, self.month, 1);
        let days = days_in_month(self.year, self.month);
        let mut day = 1_u32;
        let available = ui.available_size();
        let cell_width = ((available.x - 12.0) / 7.0).clamp(64.0, 120.0);
        let cell_height = ((available.y - 64.0) / 6.0).clamp(48.0, 85.0);

        egui::Grid::new("calendar_header")
            .num_columns(7)
            .spacing([2.0, 2.0])
            .show(ui, |ui| {
                for label in ["Mon", "Tue", "Wed", "Thu", "Fri", "Sat", "Sun"] {
                    let color = if matches!(label, "Sat") {
                        Color32::from_rgb(45, 90, 180)
                    } else if matches!(label, "Sun") {
                        Color32::from_rgb(185, 50, 50)
                    } else {
                        ui.visuals().text_color()
                    };
                    ui.allocate_ui_with_layout(
                        Vec2::new(cell_width, 40.0),
                        egui::Layout::centered_and_justified(egui::Direction::LeftToRight),
                        |ui| {
                            ui.label(RichText::new(label).size(22.0).strong().color(color));
                        },
                    );
                }
                ui.end_row();
            });

        ui.add_space(4.0);

        egui::Grid::new("calendar_grid")
            .num_columns(7)
            .spacing([2.0, 2.0])
            .show(ui, |ui| {
                for row in 0..6 {
                    for col in 0..7 {
                        let index = row * 7 + col;
                        if index < first_weekday as usize || day > days {
                            ui.add_sized([cell_width, cell_height], egui::Label::new(""));
                            continue;
                        }

                        let info = DayInfo::new(self.year, self.month, day);
                        let weekday_color = match col {
                            5 => Color32::from_rgb(45, 90, 180),
                            6 => Color32::from_rgb(185, 50, 50),
                            _ => ui.visuals().text_color(),
                        };

                        egui::Frame::group(ui.style())
                            .inner_margin(egui::Margin::same(6))
                            .show(ui, |ui| {
                                ui.set_min_size(Vec2::new(cell_width, cell_height));
                                ui.with_layout(egui::Layout::top_down(Align::Min), |ui| {
                                    ui.label(
                                        RichText::new(format!("{:02}", info.day))
                                            .size(26.0)
                                            .strong()
                                            .color(weekday_color),
                                    );
                                    ui.add_space(6.0);
                                    ui.label(
                                        RichText::new(format!("DOY {}", info.day_of_year))
                                            .size(18.0),
                                    );
                                    ui.label(RichText::new(format!("MJD {}", info.mjd)).size(18.0));
                                });
                            });

                        day += 1;
                    }
                    ui.end_row();
                }
            });
    }
}

struct DayInfo {
    day: u32,
    day_of_year: u32,
    mjd: i64,
}

impl DayInfo {
    fn new(year: i32, month: u32, day: u32) -> Self {
        Self {
            day,
            day_of_year: day_of_year(year, month, day),
            mjd: gregorian_to_mjd(year, month, day),
        }
    }
}

fn today_ymd() -> (i32, u32, u32) {
    let now = OffsetDateTime::now_local().unwrap_or_else(|_| OffsetDateTime::now_utc());
    (now.year(), now.month() as u32, now.day().into())
}

fn current_clock_strings() -> (String, String) {
    let utc_now = OffsetDateTime::now_utc();
    let jst_now = utc_now.to_offset(UtcOffset::from_hms(9, 0, 0).expect("valid JST offset"));
    (
        format_clock(jst_now, Some("JST")),
        format_clock(utc_now, Some("UT")),
    )
}

fn format_clock(datetime: OffsetDateTime, suffix: Option<&str>) -> String {
    let base = format!(
        "{:04}-{:02}-{:02} {:02}:{:02}:{:02}",
        datetime.year(),
        datetime.month() as u8,
        datetime.day(),
        datetime.hour(),
        datetime.minute(),
        datetime.second()
    );

    match suffix {
        Some(suffix) => format!("{base} {suffix}"),
        None => base,
    }
}

fn gregorian_to_mjd(year: i32, month: u32, day: u32) -> i64 {
    days_from_civil(year, month, day) + 40_587
}

fn day_of_year(year: i32, month: u32, day: u32) -> u32 {
    let mut total = day;
    let mut current = 1;
    while current < month {
        total += days_in_month(year, current);
        current += 1;
    }
    total
}

fn weekday_index(year: i32, month: u32, day: u32) -> u32 {
    (days_from_civil(year, month, day) + 3).rem_euclid(7) as u32
}

fn days_in_month(year: i32, month: u32) -> u32 {
    match month {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 if is_leap_year(year) => 29,
        2 => 28,
        _ => 0,
    }
}

fn is_leap_year(year: i32) -> bool {
    (year % 4 == 0 && year % 100 != 0) || year % 400 == 0
}

fn month_name(month: u32) -> &'static str {
    match month {
        1 => "January",
        2 => "February",
        3 => "March",
        4 => "April",
        5 => "May",
        6 => "June",
        7 => "July",
        8 => "August",
        9 => "September",
        10 => "October",
        11 => "November",
        12 => "December",
        _ => "",
    }
}

fn days_from_civil(year: i32, month: u32, day: u32) -> i64 {
    let year = year - if month <= 2 { 1 } else { 0 };
    let era = if year >= 0 { year } else { year - 399 } / 400;
    let year_of_era = year - era * 400;
    let month = month as i32;
    let day = day as i32;
    let day_of_year = (153 * (month + if month > 2 { -3 } else { 9 }) + 2) / 5 + day - 1;
    let day_of_era = year_of_era * 365 + year_of_era / 4 - year_of_era / 100 + day_of_year;
    (era * 146_097 + day_of_era - 719_468) as i64
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mjd_for_unix_epoch() {
        assert_eq!(gregorian_to_mjd(1970, 1, 1), 40_587);
    }

    #[test]
    fn doy_for_leap_year() {
        assert_eq!(day_of_year(2024, 12, 31), 366);
    }

    #[test]
    fn weekday_alignment() {
        assert_eq!(weekday_index(2025, 1, 1), 2);
    }
}
