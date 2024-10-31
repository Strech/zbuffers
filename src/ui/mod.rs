use unicode_width::UnicodeWidthChar;
use unicode_width::UnicodeWidthStr;
use zellij_tile::prelude::*;

// https://gist.github.com/fnky/458719343aabd01cfb17a3a4f7296797

// TODO: Move to another module
pub fn render_prompt(search_term: &str, colors: Colors, x: usize, y: usize) {
    let prompt = colors.green(&format!("Search:"));
    let search_term = colors.bold(&format!("{}_", search_term));

    println!(
        "\u{1b}[{};{}H\u{1b}[0m{} {}\n",
        y + 1,
        x,
        prompt,
        search_term
    );
}

// TODO: Omg
pub fn build_tab_ui_line(tab_ui_info: &TabUiInfo, colors: Colors) -> Vec<UiSpan> {
    let mut ui_spans = vec![];
    let tab_name = &tab_ui_info.name;
    // let tab_bullet_span =
    //     UiSpan::UiSpanTelescope(UiSpanTelescope::new(vec![StringAndLength::new(
    //         format!("  - "),
    //         4,
    //     )]));
    let tab_name_span =
        UiSpan::TruncatableUiSpan(TruncatableUiSpan::new(tab_name.clone(), SpanStyle::None));
    // ui_spans.push(tab_bullet_span);
    ui_spans.push(tab_name_span);
    ui_spans
}

#[derive(Debug, Default, Clone, Copy)]
pub struct Colors {
    pub palette: Palette,
}

impl Colors {
    pub fn new(palette: Palette) -> Self {
        Colors { palette }
    }
    pub fn bold(&self, text: &str) -> String {
        format!("\u{1b}[1m{}\u{1b}[22m", text)
    }

    fn color(&self, color: &PaletteColor, text: &str) -> String {
        match color {
            PaletteColor::EightBit(byte) => {
                format!("\u{1b}[38;5;{};1m{}\u{1b}[39;22m", byte, text)
            }
            PaletteColor::Rgb((r, g, b)) => {
                format!("\u{1b}[38;2;{};{};{};1m{}\u{1b}[39;22m", r, g, b, text)
            }
        }
    }
    pub fn orange(&self, text: &str) -> String {
        self.color(&self.palette.orange, text)
    }

    pub fn green(&self, text: &str) -> String {
        self.color(&self.palette.green, text)
    }

    pub fn red(&self, text: &str) -> String {
        self.color(&self.palette.red, text)
    }

    pub fn cyan(&self, text: &str) -> String {
        self.color(&self.palette.cyan, text)
    }

    pub fn magenta(&self, text: &str) -> String {
        self.color(&self.palette.magenta, text)
    }
}

#[derive(Debug, Clone)]
pub struct TabUiInfo {
    pub name: String,
    pub position: usize,
    pub is_current_tab: bool,
}

impl TabUiInfo {
    pub fn from_tab_info(tab_info: &TabInfo) -> Self {
        TabUiInfo {
            name: tab_info.name.clone(),
            position: tab_info.position,
            is_current_tab: tab_info.active,
        }
    }

    pub fn as_line_to_render(
        &self,
        _tab_index: u8,
        mut max_cols: usize,
        colors: Colors,
    ) -> LineToRender {
        let mut line_to_render = LineToRender::new(colors);
        let ui_spans = build_tab_ui_line(&self, colors);
        for span in ui_spans {
            span.render(None, &mut line_to_render, &mut max_cols);
        }
        line_to_render
    }
}

// -----------------------------------------------------------------------------

#[derive(Debug)]
pub struct ListItem {
    // TODO: Rework
    pub name: String,
    pub tab_name: Option<Vec<UiSpan>>,
    colors: Colors,
}

impl ListItem {
    pub fn line_count(&self) -> usize {
        let mut line_count = 0;
        if self.tab_name.is_some() {
            line_count += 1
        };
        line_count
    }

    pub fn render(&self, indices: Option<Vec<usize>>, max_cols: usize) -> Vec<LineToRender> {
        let mut lines_to_render = vec![];
        if let Some(tab_name) = &self.tab_name {
            let indices = indices.clone();
            let mut line_to_render = LineToRender::new(self.colors);
            let mut remaining_cols = max_cols;

            for span in tab_name {
                span.render(
                    indices
                        .clone()
                        .map(|i| (SpanStyle::ForegroundBold(self.colors.palette.magenta), i)),
                    &mut line_to_render,
                    &mut remaining_cols,
                );
            }

            lines_to_render.push(line_to_render);
        }

        lines_to_render
    }
}

// -----------------------------------------------------------------------------

#[derive(Debug, Default, Clone)]
pub struct StringAndLength {
    pub string: String,
    pub length: usize,
}

impl StringAndLength {
    pub fn new(string: String, length: usize) -> Self {
        StringAndLength { string, length }
    }
}

// -----------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub struct LineToRender {
    line: String,
    is_selected: bool,
    truncated_result_count: usize,
    colors: Colors,
}

impl LineToRender {
    pub fn new(colors: Colors) -> Self {
        LineToRender {
            line: String::default(),
            is_selected: false,
            truncated_result_count: 0,
            colors,
        }
    }

    pub fn append(&mut self, to_append: &str) {
        self.line.push_str(to_append)
    }

    pub fn make_selected_as_search(&mut self, add_arrows: bool) {
        self.is_selected = true;
        let arrows = if add_arrows {
            self.colors.magenta(" <↓↑> ")
        } else {
            "      ".to_owned()
        };
        match self.colors.palette.bg {
            PaletteColor::EightBit(byte) => {
                self.line = format!(
                    "\u{1b}[48;5;{byte}m\u{1b}[K\u{1b}[48;5;{byte}m{arrows}{}",
                    self.line
                );
            }
            PaletteColor::Rgb((r, g, b)) => {
                self.line = format!(
                    "\u{1b}[48;2;{};{};{}m\u{1b}[K\u{1b}[48;2;{};{};{}m{arrows}{}",
                    r, g, b, r, g, b, self.line
                );
            }
        }
    }

    pub fn make_selected(&mut self, add_arrows: bool) {
        self.is_selected = true;
        let arrows = if add_arrows {
            self.colors.magenta("<←↓↑→>")
        } else {
            "      ".to_owned()
        };
        match self.colors.palette.bg {
            PaletteColor::EightBit(byte) => {
                self.line = format!(
                    "\u{1b}[48;5;{byte}m\u{1b}[K\u{1b}[48;5;{byte}m{arrows}{}",
                    self.line
                );
            }
            PaletteColor::Rgb((r, g, b)) => {
                self.line = format!(
                    "\u{1b}[48;2;{};{};{}m\u{1b}[K\u{1b}[48;2;{};{};{}m{arrows}{}",
                    r, g, b, r, g, b, self.line
                );
            }
        }
    }

    pub fn render(&self) -> String {
        let mut line = self.line.clone();

        let more = if self.truncated_result_count > 0 {
            self.colors
                .red(&format!(" [+{}]", self.truncated_result_count))
        } else {
            String::new()
        };

        line.push_str(&more);
        if self.is_selected {
            self.line.clone()
        } else {
            format!("\u{1b}[49m      {}", line)
        }
    }

    pub fn add_truncated_results(&mut self, result_count: usize) {
        self.truncated_result_count += result_count;
    }
}

// -----------------------------------------------------------------------------

#[derive(Debug)]
pub enum UiSpan {
    UiSpanTelescope(UiSpanTelescope),
    TruncatableUiSpan(TruncatableUiSpan),
}

impl UiSpan {
    pub fn render(
        &self,
        indices: Option<(SpanStyle, Vec<usize>)>,
        line_to_render: &mut LineToRender,
        remaining_cols: &mut usize,
    ) {
        match self {
            UiSpan::UiSpanTelescope(ui_span_telescope) => {
                ui_span_telescope.render(line_to_render, remaining_cols)
            }
            UiSpan::TruncatableUiSpan(truncatable_ui_span) => {
                truncatable_ui_span.render(indices, line_to_render, remaining_cols)
            }
        }
    }
}

#[allow(dead_code)] // in the future this will be moved to be its own component
#[derive(Debug)]
pub enum SpanStyle {
    None,
    Bold,
    Foreground(PaletteColor),
    ForegroundBold(PaletteColor),
    Background(PaletteColor),
}

impl SpanStyle {
    pub fn style_string(&self, to_style: &str) -> String {
        match self {
            SpanStyle::None => to_style.to_owned(),
            SpanStyle::Bold => format!("\u{1b}[1m{}\u{1b}[22m", to_style),
            SpanStyle::Foreground(color) => match color {
                PaletteColor::EightBit(byte) => {
                    format!("\u{1b}[38;5;{byte}m{}\u{1b}[39m", to_style)
                }
                PaletteColor::Rgb((r, g, b)) => {
                    format!("\u{1b}[38;2;{};{};{}m{}\u{1b}[39m", r, g, b, to_style)
                }
            },
            SpanStyle::Background(color) => match color {
                PaletteColor::EightBit(byte) => {
                    format!("\u{1b}[48;5;{byte}m{}\u{1b}[39m", to_style)
                }
                PaletteColor::Rgb((r, g, b)) => {
                    format!("\u{1b}[48;2;{};{};{}m{}\u{1b}[39m", r, g, b, to_style)
                }
            },
            SpanStyle::ForegroundBold(color) => match color {
                PaletteColor::EightBit(byte) => {
                    format!("\u{1b}[38;5;{byte};1m{}\u{1b}[39;22m", to_style)
                }
                PaletteColor::Rgb((r, g, b)) => {
                    format!("\u{1b}[38;2;{};{};{};1m{}\u{1b}[39;22m", r, g, b, to_style)
                }
            },
        }
    }
}

impl Default for SpanStyle {
    fn default() -> Self {
        SpanStyle::None
    }
}

#[derive(Debug, Default)]
pub struct TruncatableUiSpan {
    text: String,
    style: SpanStyle,
}

impl TruncatableUiSpan {
    pub fn new(text: String, style: SpanStyle) -> Self {
        TruncatableUiSpan { text, style }
    }
    pub fn render(
        &self,
        indices: Option<(SpanStyle, Vec<usize>)>,
        line_to_render: &mut LineToRender,
        remaining_cols: &mut usize,
    ) {
        let mut rendered = String::new();
        let truncated = if *remaining_cols >= self.text.width() {
            self.text.clone()
        } else {
            let mut truncated = String::new();
            for character in self.text.chars() {
                if truncated.width() + character.width().unwrap_or(0) <= *remaining_cols {
                    truncated.push(character);
                } else {
                    break;
                }
            }
            truncated
        };
        match indices {
            Some((index_style, indices)) => {
                for (i, character) in truncated.chars().enumerate() {
                    // TODO: optimize this by splitting the string up by its indices and only pushing those
                    // chu8nks
                    if indices.contains(&i) {
                        rendered.push_str(&index_style.style_string(&character.to_string()));
                    } else {
                        rendered.push_str(&self.style.style_string(&character.to_string()));
                    }
                }
            }
            None => {
                rendered.push_str(&self.style.style_string(&truncated));
            }
        }
        *remaining_cols = remaining_cols.saturating_sub(truncated.width());
        line_to_render.append(&rendered);
    }
}

#[derive(Debug, Default)]
pub struct UiSpanTelescope(Vec<StringAndLength>);

impl UiSpanTelescope {
    pub fn new(string_and_lengths: Vec<StringAndLength>) -> Self {
        UiSpanTelescope(string_and_lengths)
    }
    pub fn render(&self, line_to_render: &mut LineToRender, remaining_cols: &mut usize) {
        for string_and_length in &self.0 {
            if string_and_length.length < *remaining_cols {
                line_to_render.append(&string_and_length.string);
                *remaining_cols -= string_and_length.length;
                break;
            }
        }
    }
}
