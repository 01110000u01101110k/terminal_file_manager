use std::time::Duration;
use std::time::Instant;

pub struct FixingDoubleClick {
    pub allowable_time_between_clicks: Duration,
    pub captured_first_click: bool,
    pub captured_second_click: bool,
    pub result_time_between_clicks: Option<Instant>
}

impl FixingDoubleClick {
    pub fn new() -> Self {
        Self {
            allowable_time_between_clicks: Duration::from_millis(500),
            captured_first_click: false,
            captured_second_click: false,
            result_time_between_clicks: None
        }
    }

    pub fn first_click(&mut self) {
        self.result_time_between_clicks = Some(Instant::now());
        self.captured_first_click = true;
    }

    pub fn second_click(&mut self) {
        self.captured_second_click = true;
    }

    pub fn released_both_clicks(&mut self) {
        self.captured_second_click = false;
        self.captured_first_click = false;
        self.result_time_between_clicks = None;
    }

    pub fn is_it_expired_time_between_clicks(&mut self) -> bool {
        match self.result_time_between_clicks {
            Some(result_time) => {
                if result_time.elapsed() <= self.allowable_time_between_clicks {
                    return false;
                } else {
                    return true;
                }
            },
            None => {
                return true;
            }
        }
    }

    pub fn check_it_is_double_click(&mut self) -> bool {
        if self.is_it_expired_time_between_clicks() {
            self.released_both_clicks();

            return false;
        }

        if self.captured_first_click && self.captured_second_click {
            self.released_both_clicks();

            return true;
        }

        return false;
    }
}