use lopdf::Document;
use chrono::{NaiveDate, Duration};
use regex::Regex;
use std::collections::HashMap;
use std::fs;

#[derive(Debug)]
struct Holiday {
    name: String,
    dates: Vec<String>,
}

fn main() {
    // Load PDF file
    let path = "calendars/calendar-2024-2025.pdf";
    let pdf_data = fs::read(path).expect("Failed to read PDF file");

    // Parse PDF
    let document = Document::load_mem(&pdf_data).expect("Failed to load PDF document");

    // Local variables for function
    let mut events: HashMap<String, String> = HashMap::new();
    let mut holidays: HashMap<String, Holiday> = HashMap::new();
    let date_pattern = Regex::new(r"(?P<day>\w+), (?P<month>\w+ \d{1,2}), (?P<year>\d{4})").unwrap();
    let date_range_pattern = Regex::new(r"(?P<holiday>.*?)(?P<start>\w+), (?P<month>\w+ \d{1,2}), (?P<year>\d{4}) through (?P<end>\w+), (?P<end_month>\w+ \d{1,2}), (?P<end_year>\d{4})").unwrap();
    let mut inside_holiday_section = false;

    // Extract text from PDF
    for (page_id, _) in document.get_pages() {
        if let Ok(page_text) = document.extract_text(&[page_id]) {
            for line in page_text.lines() {

                // Store school starts date
                if line.contains("School Begins") {
                    if let Some(date) = find_date(&line, &date_pattern) {
                        events.insert("School Begins".to_string(), date);
                    }
                }

                // Store school ends date
                if line.contains("School Ends") {
                    if let Some(date) = find_date(&line, &date_pattern) {
                        events.insert("School Ends".to_string(), date);
                    }
                }

                // Check if in holiday section
                if line.contains("Holidays and Other Days Schools Closed for Student Attendance") {
                    inside_holiday_section = true;
                    continue;
                } else if line.contains("Senior High School Parent/Teacher Conference Schedule") ||
                          line.contains("Junior High School Parent/Teacher Conference Schedule") ||
                          line.contains("Elementary School SEP Conference Schedule") ||
                          line.contains("Beginning and Ending of Terms") {
                    inside_holiday_section = false;
                    break; // Exit if we've left the holiday section
                }

                // Store holiday dates
                if inside_holiday_section {
                    if let Some(holiday_name) = extract_holiday_name(&line) {
                        let dates = extract_holiday_dates(&line, &date_pattern, &date_range_pattern);
                        holidays.insert(holiday_name.clone(), Holiday { name: holiday_name, dates });
                    }
                }
            }
        }
    }

    // Print all events and their dates
    for (event, date) in &events {
        println!("Event: {}, Date: {}", event, date);
    }

    // Print all holidays and their dates
    for (name, holiday) in &holidays {
        println!("Holiday: {}, Dates: {:?}", name, holiday.dates);
    }
}

// Helper function to find a date in a line
fn find_date(line: &str, date_pattern: &Regex) -> Option<String> {
    if let Some(captures) = date_pattern.captures(line) {
        Some(captures[0].to_string())
    } else {
        None
    }
}

// Helper function to extract holiday names
fn extract_holiday_name(line: &str) -> Option<String> {
  line.split('.').next().map(|s| s.trim().to_string())
}

// Helper function to extract holiday dates
fn extract_holiday_dates(line: &str, date_pattern: &Regex, date_range_pattern: &Regex) -> Option<Holiday> {
  // Check for holiday name and associated dates
  let mut holiday_name = String::new();
  let mut dates = Vec::new();

  // Look for the holiday name and date range in the line
  if let Some(captures) = date_range_pattern.captures(line) {
      holiday_name = captures[0].trim().to_string(); // You might want to refine this if there's more text before the date
      let start_date_str = format!("{}, {} {}", &captures[1], &captures[2], &captures[3]);
      let end_date_str = format!("{}, {} {}", &captures[4], &captures[5], &captures[6]);

      if let Ok(start_date) = NaiveDate::parse_from_str(&start_date_str, "%A, %B %d %Y") {
          if let Ok(end_date) = NaiveDate::parse_from_str(&end_date_str, "%A, %B %d %Y") {
              let mut current = start_date;
              while current <= end_date {
                  dates.push(current.format("%B %d, %Y").to_string());
                  current = current + Duration::days(1);
              }
          }
      }
  } else {
      // Handle individual dates if there's no range
      for cap in date_pattern.captures_iter(line) {
          holiday_name = cap[0].trim().to_string(); // Capture the holiday name first
          let date_part = format!("{}, {} {}", &cap["day"], &cap["month"], &cap["year"]);

          if let Ok(date) = NaiveDate::parse_from_str(&date_part, "%A, %B %d %Y") {
              dates.push(date.format("%B %d, %Y").to_string());
          }
      }
  }

  // If we captured a holiday name and dates, return them
  if !holiday_name.is_empty() && !dates.is_empty() {
      return Some(Holiday { name: holiday_name, dates });
  }

  None
}