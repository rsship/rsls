use std::collections::HashMap;
use std::fmt;
use std::str::FromStr;

#[derive(Default, Debug)]
pub struct Rfc2822 {
    day: u32,
    month: String,
    year: u32,
    hour: u32,
    minute: u32,
    second: u32,
    offset: i32,
}

impl FromStr for Rfc2822 {
    //NOTE: standard string version is "Fri, 14 Jul 2017 02:40:00 +0000";
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let elems = s.split_whitespace().into_iter().collect::<Vec<&str>>();

        if elems.len() != 6 {
            return Err("Invalid RFC2228 Date format".to_string());
        }

        let day = elems[1].parse().unwrap();
        let month = elems[2];
        let year = elems[3].parse().unwrap();
        let time = elems[4].split(":").collect::<Vec<_>>();
        let offset = elems[5];

        let hour = time[0].parse::<u32>().unwrap();
        let min = time[1].parse::<u32>().unwrap();
        let secs = time[2].parse::<u32>().unwrap();

        let offset = if offset.len() == 5 {
            let sign = &offset[0..1];
            let hours = offset[1..3].parse::<i32>().unwrap();
            let mins = offset[3..5].parse::<i32>().unwrap();

            if sign == "-" {
                -1 * (hours * 60 + mins)
            } else {
                hours * 60 + mins
            }
        } else {
            return Err("Invalid RFC2228 Date format".to_string());
        };

        Ok(Rfc2822 {
            day,
            month: month.to_string(),
            year,
            hour,
            minute: min,
            second: secs,
            offset,
        })
    }
}

impl fmt::Display for Rfc2822 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        write!(
            f,
            "{}, {:02} {:02} {:04} {:02}:{:02}:{:02} {:05}",
            self.day_of_week_gregorian(),
            self.day,
            self.month,
            self.year,
            self.hour,
            self.minute,
            self.second,
            self.offset,
        )
    }
}

impl Rfc2822 {
    fn day_of_week_gregorian(&self) -> &str {
        let mut days = HashMap::<u32, &str>::new();
        let mut months = HashMap::<&str, u32>::new();

        days.insert(0, "SaturDay");
        days.insert(1, "Sunday");
        days.insert(2, "Monday");
        days.insert(3, "Tuesday");
        days.insert(4, "Wednesday");
        days.insert(5, "Thursday");
        days.insert(6, "Friday");

        months.insert("Mar", 3);
        months.insert("Apr", 4);
        months.insert("May", 5);
        months.insert("Jun", 6);
        months.insert("Jul", 7);
        months.insert("Aug", 8);
        months.insert("Sep", 9);
        months.insert("Oct", 10);
        months.insert("Nov", 11);
        months.insert("Dec", 12);
        months.insert("Jan", 13);
        months.insert("Fab", 14);

        let k = self.year % 100;
        let j = self.year / 100;

        //NOTE: gregorian calander day of the week implementation based on Zeller's congruence
        // https://en.wikipedia.org/wiki/Zeller%27s_congruence
        let m = months.get(self.month.as_str()).unwrap();
        let h = (self.day + (13 * (m + 1) / 5) + k + (k / 4) + (j / 4) - 2 * j) % 7;

        days.get(&h).unwrap()
    }
}
