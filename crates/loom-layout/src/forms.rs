//! HTML Forms support for Loom
//!
//! Phase L12: Forms
//! - Input elements: text, password, checkbox, radio, textarea, select
//! - Form state management with focus and cursor
//! - Keyboard input integration
//! - Form submission (GET/POST)
//! - Basic validation (required, email)

use alloc::string::String;
use alloc::vec::Vec;
use alloc::collections::BTreeMap;
use alloc::format;

/// Input element types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InputType {
    Text,
    Password,
    Checkbox,
    Radio,
    Email,
    Number,
    Submit,
    Button,
    Hidden,
}

impl InputType {
    pub fn from_attr(attr: &str) -> Self {
        match attr.to_lowercase().as_str() {
            "password" => InputType::Password,
            "checkbox" => InputType::Checkbox,
            "radio" => InputType::Radio,
            "email" => InputType::Email,
            "number" => InputType::Number,
            "submit" => InputType::Submit,
            "button" => InputType::Button,
            "hidden" => InputType::Hidden,
            _ => InputType::Text,
        }
    }
}

/// A form input element's state
#[derive(Debug, Clone)]
pub struct InputState {
    /// Element ID or name
    pub name: String,
    /// Input type
    pub input_type: InputType,
    /// Current value
    pub value: String,
    /// For checkbox/radio: whether checked
    pub checked: bool,
    /// Placeholder text
    pub placeholder: String,
    /// Whether field is required
    pub required: bool,
    /// Whether field is currently focused
    pub focused: bool,
    /// Cursor position (for text inputs)
    pub cursor_pos: usize,
    /// Selection start (if any)
    pub selection_start: Option<usize>,
    /// Selection end (if any)
    pub selection_end: Option<usize>,
    /// Whether field has validation error
    pub has_error: bool,
    /// Error message
    pub error_message: Option<String>,
    /// For radio: group name
    pub radio_group: Option<String>,
    /// For select: options
    pub options: Vec<SelectOption>,
    /// For select: selected index
    pub selected_index: Option<usize>,
    /// Whether element is disabled
    pub disabled: bool,
    /// Whether element is readonly
    pub readonly: bool,
    /// Maximum length (for text inputs)
    pub max_length: Option<usize>,
}

/// Option for select elements
#[derive(Debug, Clone)]
pub struct SelectOption {
    pub value: String,
    pub label: String,
    pub selected: bool,
}

impl InputState {
    pub fn new(name: &str, input_type: InputType) -> Self {
        Self {
            name: name.to_string(),
            input_type,
            value: String::new(),
            checked: false,
            placeholder: String::new(),
            required: false,
            focused: false,
            cursor_pos: 0,
            selection_start: None,
            selection_end: None,
            has_error: false,
            error_message: None,
            radio_group: None,
            options: Vec::new(),
            selected_index: None,
            disabled: false,
            readonly: false,
            max_length: None,
        }
    }
    
    pub fn with_placeholder(mut self, placeholder: &str) -> Self {
        self.placeholder = placeholder.to_string();
        self
    }
    
    pub fn with_required(mut self, required: bool) -> Self {
        self.required = required;
        self
    }
    
    pub fn with_value(mut self, value: &str) -> Self {
        self.value = value.to_string();
        self.cursor_pos = value.len();
        self
    }
    
    pub fn with_checked(mut self, checked: bool) -> Self {
        self.checked = checked;
        self
    }
    
    pub fn with_radio_group(mut self, group: &str) -> Self {
        self.radio_group = Some(group.to_string());
        self
    }
    
    pub fn with_max_length(mut self, max: usize) -> Self {
        self.max_length = Some(max);
        self
    }
    
    /// Insert a character at cursor position
    pub fn insert_char(&mut self, ch: char) {
        if self.readonly || self.disabled {
            return;
        }
        
        // Check max length
        if let Some(max) = self.max_length {
            if self.value.len() >= max {
                return;
            }
        }
        
        // Clear any selection
        if self.selection_start.is_some() {
            self.delete_selection();
        }
        
        // Insert character
        if self.cursor_pos > self.value.len() {
            self.cursor_pos = self.value.len();
        }
        self.value.insert(self.cursor_pos, ch);
        self.cursor_pos += 1;
        
        // Clear error on input
        self.clear_error();
    }
    
    /// Delete character before cursor (backspace)
    pub fn backspace(&mut self) {
        if self.readonly || self.disabled {
            return;
        }
        
        if self.selection_start.is_some() {
            self.delete_selection();
        } else if self.cursor_pos > 0 {
            self.cursor_pos -= 1;
            self.value.remove(self.cursor_pos);
        }
        
        self.clear_error();
    }
    
    /// Delete character after cursor (delete key)
    pub fn delete(&mut self) {
        if self.readonly || self.disabled {
            return;
        }
        
        if self.selection_start.is_some() {
            self.delete_selection();
        } else if self.cursor_pos < self.value.len() {
            self.value.remove(self.cursor_pos);
        }
        
        self.clear_error();
    }
    
    /// Move cursor left
    pub fn cursor_left(&mut self, shift: bool) {
        if self.cursor_pos > 0 {
            if shift {
                self.start_selection();
                self.cursor_pos -= 1;
                self.selection_end = Some(self.cursor_pos);
            } else {
                self.cursor_pos -= 1;
                self.clear_selection();
            }
        }
    }
    
    /// Move cursor right
    pub fn cursor_right(&mut self, shift: bool) {
        if self.cursor_pos < self.value.len() {
            if shift {
                self.start_selection();
                self.cursor_pos += 1;
                self.selection_end = Some(self.cursor_pos);
            } else {
                self.cursor_pos += 1;
                self.clear_selection();
            }
        }
    }
    
    /// Move cursor to start
    pub fn cursor_home(&mut self, shift: bool) {
        if shift {
            self.start_selection();
            self.cursor_pos = 0;
            self.selection_end = Some(0);
        } else {
            self.cursor_pos = 0;
            self.clear_selection();
        }
    }
    
    /// Move cursor to end
    pub fn cursor_end(&mut self, shift: bool) {
        let end = self.value.len();
        if shift {
            self.start_selection();
            self.cursor_pos = end;
            self.selection_end = Some(end);
        } else {
            self.cursor_pos = end;
            self.clear_selection();
        }
    }
    
    fn start_selection(&mut self) {
        if self.selection_start.is_none() {
            self.selection_start = Some(self.cursor_pos);
            self.selection_end = Some(self.cursor_pos);
        }
    }
    
    fn clear_selection(&mut self) {
        self.selection_start = None;
        self.selection_end = None;
    }
    
    fn delete_selection(&mut self) {
        if let (Some(start), Some(end)) = (self.selection_start, self.selection_end) {
            let (start, end) = if start <= end { (start, end) } else { (end, start) };
            self.value.drain(start..end);
            self.cursor_pos = start;
            self.clear_selection();
        }
    }
    
    /// Toggle checkbox
    pub fn toggle_checkbox(&mut self) {
        if self.input_type == InputType::Checkbox && !self.disabled && !self.readonly {
            self.checked = !self.checked;
        }
    }
    
    /// Check radio button
    pub fn check_radio(&mut self) {
        if self.input_type == InputType::Radio && !self.disabled && !self.readonly {
            self.checked = true;
        }
    }
    
    /// Get display value (masked for password)
    pub fn display_value(&self) -> String {
        match self.input_type {
            InputType::Password => "•".repeat(self.value.len()),
            _ => self.value.clone(),
        }
    }
    
    /// Get the value to submit
    pub fn submit_value(&self) -> Option<String> {
        if self.disabled {
            return None;
        }
        
        match self.input_type {
            InputType::Checkbox => {
                if self.checked {
                    if self.value.is_empty() {
                        Some("on".to_string())
                    } else {
                        Some(self.value.clone())
                    }
                } else {
                    None
                }
            }
            InputType::Radio => {
                if self.checked {
                    Some(self.value.clone())
                } else {
                    None
                }
            }
            InputType::Submit | InputType::Button => None,
            _ => Some(self.value.clone()),
        }
    }
    
    /// Validate the input
    pub fn validate(&mut self) -> bool {
        self.has_error = false;
        self.error_message = None;
        
        // Required check
        if self.required {
            let empty = match self.input_type {
                InputType::Checkbox => !self.checked,
                _ => self.value.trim().is_empty(),
            };
            
            if empty {
                self.has_error = true;
                self.error_message = Some("This field is required".to_string());
                return false;
            }
        }
        
        // Email validation
        if self.input_type == InputType::Email && !self.value.is_empty() {
            if !self.is_valid_email() {
                self.has_error = true;
                self.error_message = Some("Please enter a valid email address".to_string());
                return false;
            }
        }
        
        // Number validation
        if self.input_type == InputType::Number && !self.value.is_empty() {
            if self.value.parse::<f64>().is_err() {
                self.has_error = true;
                self.error_message = Some("Please enter a valid number".to_string());
                return false;
            }
        }
        
        true
    }
    
    fn is_valid_email(&self) -> bool {
        // Basic email regex: local@domain.tld
        let parts: Vec<&str> = self.value.split('@').collect();
        if parts.len() != 2 {
            return false;
        }
        let local = parts[0];
        let domain = parts[1];
        
        if local.is_empty() || domain.is_empty() {
            return false;
        }
        
        // Domain must have at least one dot
        if !domain.contains('.') {
            return false;
        }
        
        // Domain part after last dot must be at least 2 chars
        let domain_parts: Vec<&str> = domain.split('.').collect();
        if let Some(tld) = domain_parts.last() {
            if tld.len() < 2 {
                return false;
            }
        }
        
        true
    }
    
    fn clear_error(&mut self) {
        self.has_error = false;
        self.error_message = None;
    }
    
    /// Select an option (for select elements)
    pub fn select_option(&mut self, index: usize) {
        if index < self.options.len() {
            // Clear previous selection
            for (i, opt) in self.options.iter_mut().enumerate() {
                opt.selected = i == index;
            }
            self.selected_index = Some(index);
            self.value = self.options[index].value.clone();
        }
    }
}

/// A form containing multiple inputs
#[derive(Debug, Clone)]
pub struct Form {
    /// Form ID
    pub id: String,
    /// Action URL
    pub action: String,
    /// Method (GET or POST)
    pub method: FormMethod,
    /// Input elements by name
    pub inputs: BTreeMap<String, InputState>,
    /// Focused input name
    pub focused_input: Option<String>,
    /// Submit button name
    pub submit_button: Option<String>,
}

/// HTTP method for form submission
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FormMethod {
    Get,
    Post,
}

impl FormMethod {
    pub fn from_attr(attr: &str) -> Self {
        match attr.to_uppercase().as_str() {
            "POST" => FormMethod::Post,
            _ => FormMethod::Get,
        }
    }
}

impl Form {
    pub fn new(id: &str) -> Self {
        Self {
            id: id.to_string(),
            action: String::new(),
            method: FormMethod::Get,
            inputs: BTreeMap::new(),
            focused_input: None,
            submit_button: None,
        }
    }
    
    pub fn with_action(mut self, action: &str) -> Self {
        self.action = action.to_string();
        self
    }
    
    pub fn with_method(mut self, method: FormMethod) -> Self {
        self.method = method;
        self
    }
    
    pub fn add_input(&mut self, input: InputState) {
        let name = input.name.clone();
        self.inputs.insert(name, input);
    }
    
    /// Get mutable input by name
    pub fn get_input_mut(&mut self, name: &str) -> Option<&mut InputState> {
        self.inputs.get_mut(name)
    }
    
    /// Get input by name
    pub fn get_input(&self, name: &str) -> Option<&InputState> {
        self.inputs.get(name)
    }
    
    /// Focus an input by name
    pub fn focus_input(&mut self, name: &str) {
        // Blur current focused input
        if let Some(current) = &self.focused_input {
            if let Some(input) = self.inputs.get_mut(current) {
                input.focused = false;
            }
        }
        
        // Focus new input
        if let Some(input) = self.inputs.get_mut(name) {
            input.focused = true;
            self.focused_input = Some(name.to_string());
        }
    }
    
    /// Blur all inputs
    pub fn blur_all(&mut self) {
        if let Some(current) = &self.focused_input {
            if let Some(input) = self.inputs.get_mut(current) {
                input.focused = false;
            }
        }
        self.focused_input = None;
    }
    
    /// Get currently focused input
    pub fn focused_input_mut(&mut self) -> Option<&mut InputState> {
        self.focused_input.as_ref()
            .and_then(|name| self.inputs.get_mut(name))
    }
    
    /// Get currently focused input (immutable)
    pub fn focused_input(&self) -> Option<&InputState> {
        self.focused_input.as_ref()
            .and_then(|name| self.inputs.get(name))
    }
    
    /// Cycle focus to next input
    pub fn focus_next(&mut self) {
        let names: Vec<String> = self.inputs.keys().cloned().collect();
        if names.is_empty() {
            return;
        }
        
        let next_idx = if let Some(current) = &self.focused_input {
            if let Some(idx) = names.iter().position(|n| n == current) {
                (idx + 1) % names.len()
            } else {
                0
            }
        } else {
            0
        };
        
        self.focus_input(&names[next_idx]);
    }
    
    /// Cycle focus to previous input
    pub fn focus_prev(&mut self) {
        let names: Vec<String> = self.inputs.keys().cloned().collect();
        if names.is_empty() {
            return;
        }
        
        let prev_idx = if let Some(current) = &self.focused_input {
            if let Some(idx) = names.iter().position(|n| n == current) {
                if idx == 0 {
                    names.len() - 1
                } else {
                    idx - 1
                }
            } else {
                0
            }
        } else {
            0
        };
        
        self.focus_input(&names[prev_idx]);
    }
    
    /// Validate all inputs
    pub fn validate(&mut self) -> bool {
        let mut all_valid = true;
        for input in self.inputs.values_mut() {
            if !input.validate() {
                all_valid = false;
            }
        }
        all_valid
    }
    
    /// Collect form data for submission
    pub fn collect_data(&self) -> BTreeMap<String, String> {
        let mut data = BTreeMap::new();
        for (name, input) in &self.inputs {
            if let Some(value) = input.submit_value() {
                data.insert(name.clone(), value);
            }
        }
        data
    }
    
    /// Encode form data as application/x-www-form-urlencoded
    pub fn encode_urlencoded(&self) -> String {
        let data = self.collect_data();
        let mut pairs = Vec::new();
        
        for (key, value) in data {
            let encoded_key = urlencode(&key);
            let encoded_value = urlencode(&value);
            pairs.push(format!("{}={}", encoded_key, encoded_value));
        }
        
        pairs.join("&")
    }
    
    /// Get submission URL with query params for GET requests
    pub fn get_submission_url(&self) -> String {
        if self.method == FormMethod::Get && !self.inputs.is_empty() {
            let query = self.encode_urlencoded();
            if query.is_empty() {
                self.action.clone()
            } else if self.action.contains('?') {
                format!("{}&{}", self.action, query)
            } else {
                format!("{}?{}", self.action, query)
            }
        } else {
            self.action.clone()
        }
    }
    
    /// Get radio group inputs
    pub fn get_radio_group(&self, group_name: &str) -> Vec<&InputState> {
        self.inputs.values()
            .filter(|input| {
                input.input_type == InputType::Radio &&
                input.radio_group.as_deref() == Some(group_name)
            })
            .collect()
    }
    
    /// Uncheck all radios in a group
    pub fn uncheck_radio_group(&mut self, group_name: &str) {
        for input in self.inputs.values_mut() {
            if input.input_type == InputType::Radio &&
               input.radio_group.as_deref() == Some(group_name) {
                input.checked = false;
            }
        }
    }
    
    /// Check a specific radio button in a group
    pub fn check_radio(&mut self, group_name: &str, value: &str) {
        // First uncheck all in group
        self.uncheck_radio_group(group_name);
        
        // Check the selected one
        for input in self.inputs.values_mut() {
            if input.input_type == InputType::Radio &&
               input.radio_group.as_deref() == Some(group_name) &&
               input.value == value {
                input.checked = true;
                break;
            }
        }
    }
}

/// Simple URL encoding for form data
fn urlencode(s: &str) -> String {
    let mut result = String::with_capacity(s.len() * 3);
    for byte in s.bytes() {
        match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                result.push(byte as char);
            }
            b' ' => result.push('+'),
            _ => {
                result.push('%');
                result.push_str(&format!("{:02X}", byte));
            }
        }
    }
    result
}

/// Form manager for the page
#[derive(Debug, Default)]
pub struct FormManager {
    /// Forms by ID
    pub forms: BTreeMap<String, Form>,
    /// Currently focused form
    pub active_form: Option<String>,
    /// Cursor blink state (toggled by render loop)
    pub cursor_visible: bool,
    /// Last cursor blink time
    pub last_blink_time: u64,
    /// Blink interval in ms
    pub blink_interval: u64,
}

impl FormManager {
    pub fn new() -> Self {
        Self {
            forms: BTreeMap::new(),
            active_form: None,
            cursor_visible: true,
            last_blink_time: 0,
            blink_interval: 500, // 500ms blink
        }
    }
    
    /// Add a form
    pub fn add_form(&mut self, form: Form) {
        self.forms.insert(form.id.clone(), form);
    }
    
    /// Get mutable form
    pub fn get_form_mut(&mut self, id: &str) -> Option<&mut Form> {
        self.forms.get_mut(id)
    }
    
    /// Get form
    pub fn get_form(&self, id: &str) -> Option<&Form> {
        self.forms.get(id)
    }
    
    /// Set active form
    pub fn set_active_form(&mut self, id: &str) {
        self.active_form = Some(id.to_string());
    }
    
    /// Get active form
    pub fn active_form_mut(&mut self) -> Option<&mut Form> {
        self.active_form.as_ref()
            .and_then(|id| self.forms.get_mut(id))
    }
    
    /// Get active form (immutable)
    pub fn active_form(&self) -> Option<&Form> {
        self.active_form.as_ref()
            .and_then(|id| self.forms.get(id))
    }
    
    /// Get currently focused input from active form
    pub fn focused_input_mut(&mut self) -> Option<&mut InputState> {
        self.active_form_mut()
            .and_then(|form| form.focused_input_mut())
    }
    
    /// Get currently focused input (immutable)
    pub fn focused_input(&self) -> Option<&InputState> {
        self.active_form()
            .and_then(|form| form.focused_input())
    }
    
    /// Handle character input
    pub fn handle_char(&mut self, ch: char) {
        if let Some(input) = self.focused_input_mut() {
            if input.input_type != InputType::Checkbox &&
               input.input_type != InputType::Radio &&
               input.input_type != InputType::Submit &&
               input.input_type != InputType::Button {
                input.insert_char(ch);
            }
        }
    }
    
    /// Handle backspace
    pub fn handle_backspace(&mut self) {
        if let Some(input) = self.focused_input_mut() {
            input.backspace();
        }
    }
    
    /// Handle delete
    pub fn handle_delete(&mut self) {
        if let Some(input) = self.focused_input_mut() {
            input.delete();
        }
    }
    
    /// Handle cursor left
    pub fn handle_cursor_left(&mut self, shift: bool) {
        if let Some(input) = self.focused_input_mut() {
            input.cursor_left(shift);
        }
    }
    
    /// Handle cursor right
    pub fn handle_cursor_right(&mut self, shift: bool) {
        if let Some(input) = self.focused_input_mut() {
            input.cursor_right(shift);
        }
    }
    
    /// Handle cursor home
    pub fn handle_cursor_home(&mut self, shift: bool) {
        if let Some(input) = self.focused_input_mut() {
            input.cursor_home(shift);
        }
    }
    
    /// Handle cursor end
    pub fn handle_cursor_end(&mut self, shift: bool) {
        if let Some(input) = self.focused_input_mut() {
            input.cursor_end(shift);
        }
    }
    
    /// Handle tab (focus next)
    pub fn handle_tab(&mut self, shift: bool) {
        if let Some(form) = self.active_form_mut() {
            if shift {
                form.focus_prev();
            } else {
                form.focus_next();
            }
        }
    }
    
    /// Handle enter (submit form or activate button)
    pub fn handle_enter(&mut self) -> Option<FormSubmission> {
        // Check if focused input is a submit button
        if let Some(input) = self.focused_input() {
            if input.input_type == InputType::Submit {
                return self.submit_active_form();
            }
        }
        
        // Otherwise submit the form
        self.submit_active_form()
    }
    
    /// Handle escape (blur)
    pub fn handle_escape(&mut self) {
        if let Some(form) = self.active_form_mut() {
            form.blur_all();
        }
        self.active_form = None;
    }
    
    /// Handle space (toggle checkbox)
    pub fn handle_space(&mut self) {
        if let Some(input) = self.focused_input_mut() {
            if input.input_type == InputType::Checkbox {
                input.toggle_checkbox();
            }
        }
    }
    
    /// Submit the active form
    pub fn submit_active_form(&mut self) -> Option<FormSubmission> {
        let form = self.active_form_mut()?;
        
        // Validate
        if !form.validate() {
            return None;
        }
        
        let url = form.get_submission_url();
        let method = form.method;
        let body = if method == FormMethod::Post {
            Some(form.encode_urlencoded())
        } else {
            None
        };
        
        Some(FormSubmission {
            url,
            method,
            body,
        })
    }
    
    /// Toggle cursor blink
    pub fn update_cursor_blink(&mut self, current_time: u64) {
        if current_time - self.last_blink_time >= self.blink_interval {
            self.cursor_visible = !self.cursor_visible;
            self.last_blink_time = current_time;
        }
    }
    
    /// Check if cursor should be visible (for rendering)
    pub fn should_show_cursor(&self) -> bool {
        self.cursor_visible
    }
    
    /// Click on an input element
    pub fn click_input(&mut self, form_id: &str, input_name: &str) {
        // First, determine the input type without keeping a borrow
        let input_type = self.forms.get(form_id)
            .and_then(|f| f.get_input(input_name))
            .map(|input| (input.input_type, input.radio_group.clone(), input.value.clone()));
        
        if let Some((input_type, radio_group, value)) = input_type {
            if let Some(form) = self.forms.get_mut(form_id) {
                match input_type {
                    InputType::Checkbox => {
                        if let Some(input) = form.get_input_mut(input_name) {
                            input.toggle_checkbox();
                        }
                    }
                    InputType::Radio => {
                        if let Some(group) = radio_group {
                            form.check_radio(&group, &value);
                        } else {
                            if let Some(input) = form.get_input_mut(input_name) {
                                input.check_radio();
                            }
                        }
                    }
                    _ => {
                        // Focus text inputs
                        form.focus_input(input_name);
                        self.set_active_form(form_id);
                    }
                }
            }
        }
    }
}

/// A form submission request
#[derive(Debug, Clone)]
pub struct FormSubmission {
    pub url: String,
    pub method: FormMethod,
    pub body: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_input_text_entry() {
        let mut input = InputState::new("username", InputType::Text);
        
        input.insert_char('h');
        input.insert_char('i');
        assert_eq!(input.value, "hi");
        assert_eq!(input.cursor_pos, 2);
    }
    
    #[test]
    fn test_input_backspace() {
        let mut input = InputState::new("username", InputType::Text)
            .with_value("hi");
        
        input.backspace();
        assert_eq!(input.value, "h");
        assert_eq!(input.cursor_pos, 1);
    }
    
    #[test]
    fn test_input_cursor_movement() {
        let mut input = InputState::new("username", InputType::Text)
            .with_value("hello");
        
        input.cursor_home(false);
        assert_eq!(input.cursor_pos, 0);
        
        input.cursor_end(false);
        assert_eq!(input.cursor_pos, 5);
    }
    
    #[test]
    fn test_password_masking() {
        let input = InputState::new("password", InputType::Password)
            .with_value("secret");
        
        assert_eq!(input.display_value(), "••••••");
    }
    
    #[test]
    fn test_checkbox_toggle() {
        let mut input = InputState::new("agree", InputType::Checkbox);
        
        assert!(!input.checked);
        input.toggle_checkbox();
        assert!(input.checked);
        input.toggle_checkbox();
        assert!(!input.checked);
    }
    
    #[test]
    fn test_required_validation() {
        let mut input = InputState::new("username", InputType::Text)
            .with_required(true);
        
        assert!(!input.validate());
        assert!(input.has_error);
        
        input.value = "john".to_string();
        assert!(input.validate());
        assert!(!input.has_error);
    }
    
    #[test]
    fn test_email_validation() {
        let mut input = InputState::new("email", InputType::Email);
        
        // Valid emails
        input.value = "test@example.com".to_string();
        assert!(input.validate());
        
        input.value = "user@domain.org".to_string();
        assert!(input.validate());
        
        // Invalid emails
        input.value = "invalid".to_string();
        assert!(!input.validate());
        
        input.value = "@example.com".to_string();
        assert!(!input.validate());
        
        input.value = "user@".to_string();
        assert!(!input.validate());
    }
    
    #[test]
    fn test_form_focus_cycling() {
        let mut form = Form::new("login");
        form.add_input(InputState::new("username", InputType::Text));
        form.add_input(InputState::new("password", InputType::Password));
        form.add_input(InputState::new("remember", InputType::Checkbox));
        
        // Focus first
        form.focus_input("username");
        assert_eq!(form.focused_input, Some("username".to_string()));
        
        // Tab to next
        form.focus_next();
        assert_eq!(form.focused_input, Some("password".to_string()));
        
        // Tab to next
        form.focus_next();
        assert_eq!(form.focused_input, Some("remember".to_string()));
        
        // Tab wraps to first
        form.focus_next();
        assert_eq!(form.focused_input, Some("username".to_string()));
    }
    
    #[test]
    fn test_form_url_encoding() {
        let mut form = Form::new("search")
            .with_action("/search");
        
        form.add_input(InputState::new("q", InputType::Text).with_value("hello world"));
        form.add_input(InputState::new("page", InputType::Number).with_value("1"));
        
        let encoded = form.encode_urlencoded();
        assert!(encoded.contains("q=hello+world"));
        assert!(encoded.contains("page=1"));
    }
    
    #[test]
    fn test_get_submission_url() {
        let mut form = Form::new("search")
            .with_action("/search")
            .with_method(FormMethod::Get);
        
        form.add_input(InputState::new("q", InputType::Text).with_value("rust"));
        
        let url = form.get_submission_url();
        assert_eq!(url, "/search?q=rust");
    }
    
    #[test]
    fn test_radio_group() {
        let mut form = Form::new("survey");
        
        form.add_input(InputState::new("opt1", InputType::Radio)
            .with_value("a")
            .with_radio_group("question1"));
        form.add_input(InputState::new("opt2", InputType::Radio)
            .with_value("b")
            .with_radio_group("question1"));
        form.add_input(InputState::new("opt3", InputType::Radio)
            .with_value("c")
            .with_radio_group("question1"));
        
        // Check one
        form.check_radio("question1", "b");
        
        let opt1 = form.get_input("opt1").unwrap();
        let opt2 = form.get_input("opt2").unwrap();
        let opt3 = form.get_input("opt3").unwrap();
        
        assert!(!opt1.checked);
        assert!(opt2.checked);
        assert!(!opt3.checked);
    }
    
    #[test]
    fn test_max_length() {
        let mut input = InputState::new("code", InputType::Text)
            .with_max_length(5);
        
        input.insert_char('1');
        input.insert_char('2');
        input.insert_char('3');
        input.insert_char('4');
        input.insert_char('5');
        input.insert_char('6'); // Should be ignored
        
        assert_eq!(input.value, "12345");
    }
}