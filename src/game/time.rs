#[derive(Debug, Clone, PartialEq)]
pub enum Season {
    Spring,
    Summer,
    Fall,
    Winter,
}

impl Season {
    pub fn name(&self) -> &'static str {
        match self {
            Season::Spring => "Spring",
            Season::Summer => "Summer",
            Season::Fall   => "Fall",
            Season::Winter => "Winter",
        }
    }

    pub fn from_name(name: &str) -> Option<Season> {
        match name {
            "Spring" => Some(Season::Spring),
            "Summer" => Some(Season::Summer),
            "Fall"   => Some(Season::Fall),
            "Winter" => Some(Season::Winter),
            _ => None,
        }
    }

    pub fn next(&self) -> Season {
        match self {
            Season::Spring => Season::Summer,
            Season::Summer => Season::Fall,
            Season::Fall   => Season::Winter,
            Season::Winter => Season::Spring,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum TimeEvent {
    Normal,
    ForcedSleep,
    SeasonEnd,
}

#[derive(Debug, Clone)]
pub struct GameClock {
    pub day: u8,
    pub hour: u8,
    pub minute: u8,
    pub season: Season,
    pub year: u16,   // starts at 1; increments each Winter → Spring rollover
}

impl GameClock {
    pub fn new() -> Self {
        Self { day: 1, hour: 6, minute: 0, season: Season::Spring, year: 1 }
    }

    /// Advance by 10 in-game minutes. Returns a TimeEvent if something notable happens.
    pub fn tick(&mut self) -> TimeEvent {
        self.minute += 10;
        if self.minute >= 60 {
            self.minute = 0;
            self.hour += 1;
        }
        if self.hour >= 26 {
            return TimeEvent::ForcedSleep;
        }
        TimeEvent::Normal
    }

    /// Advance to the next day. Cycles season after day 28. Returns SeasonEnd on transition.
    /// Also increments `year` when Winter rolls over to Spring.
    pub fn advance_day(&mut self) -> TimeEvent {
        if self.day >= 28 {
            let next = self.season.next();
            if next == Season::Spring {
                self.year = self.year.saturating_add(1);
            }
            self.season = next;
            self.day = 1;
            self.hour = 6;
            self.minute = 0;
            return TimeEvent::SeasonEnd;
        }
        self.day += 1;
        self.hour = 6;
        self.minute = 0;
        TimeEvent::Normal
    }

    pub fn display_time(&self) -> String {
        let display_hour = if self.hour > 12 { self.hour - 12 } else { self.hour };
        let am_pm = if self.hour < 12 { "AM" } else { "PM" };
        format!("{:02}:{:02} {}", display_hour, self.minute, am_pm)
    }

    pub fn display_date(&self) -> String {
        format!("{} Day {} · Yr {}", self.season.name(), self.day, self.year)
    }
}

impl Default for GameClock {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tick_advances_ten_minutes() {
        let mut clock = GameClock::new();
        assert_eq!(clock.minute, 0);
        clock.tick();
        assert_eq!(clock.minute, 10);
    }

    #[test]
    fn tick_wraps_minute_to_next_hour() {
        let mut clock = GameClock::new();
        for _ in 0..6 {
            clock.tick();
        }
        assert_eq!(clock.hour, 7);
        assert_eq!(clock.minute, 0);
    }

    #[test]
    fn hour_26_triggers_forced_sleep() {
        let mut clock = GameClock::new();
        clock.hour = 25;
        clock.minute = 50;
        let event = clock.tick();
        assert_eq!(event, TimeEvent::ForcedSleep);
    }

    #[test]
    fn advance_day_increments_day() {
        let mut clock = GameClock::new();
        clock.advance_day();
        assert_eq!(clock.day, 2);
        assert_eq!(clock.hour, 6);
        assert_eq!(clock.minute, 0);
    }

    #[test]
    fn day_28_advance_returns_season_end() {
        let mut clock = GameClock::new();
        clock.day = 28;
        let event = clock.advance_day();
        assert_eq!(event, TimeEvent::SeasonEnd);
        assert_eq!(clock.day, 1);
        assert_eq!(clock.season, Season::Summer);
    }

    #[test]
    fn season_cycles_through_all_four() {
        let mut clock = GameClock::new();
        assert_eq!(clock.season, Season::Spring);
        clock.day = 28; clock.advance_day();
        assert_eq!(clock.season, Season::Summer);
        clock.day = 28; clock.advance_day();
        assert_eq!(clock.season, Season::Fall);
        clock.day = 28; clock.advance_day();
        assert_eq!(clock.season, Season::Winter);
        clock.day = 28; clock.advance_day();
        assert_eq!(clock.season, Season::Spring);
    }

    #[test]
    fn display_date_uses_current_season() {
        let mut clock = GameClock::new();
        assert_eq!(clock.display_date(), "Spring Day 1 · Yr 1");
        clock.season = Season::Summer;
        clock.day = 5;
        assert_eq!(clock.display_date(), "Summer Day 5 · Yr 1");
    }

    #[test]
    fn year_increments_on_winter_to_spring() {
        let mut clock = GameClock::new();
        // Fast-forward through Spring, Summer, Fall, Winter
        for _ in 0..4 {
            clock.day = 28;
            clock.advance_day();
        }
        assert_eq!(clock.year, 2);
        assert_eq!(clock.season, Season::Spring);
    }

    #[test]
    fn year_does_not_increment_on_other_season_ends() {
        let mut clock = GameClock::new();
        clock.day = 28;
        clock.advance_day(); // Spring → Summer
        assert_eq!(clock.year, 1);
    }

    #[test]
    fn display_time_formats_correctly() {
        let clock = GameClock::new();
        assert_eq!(clock.display_time(), "06:00 AM");
    }
}
