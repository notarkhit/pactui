use ratatui::style::Color;

#[derive(Debug, Clone)]
pub struct Theme {
    pub name: &'static str,
    pub bg: Color,
    pub fg: Color,
    pub border: Color,
    pub accent: Color,
    pub green: Color,
    pub red: Color,
    pub yellow: Color,
    pub blue: Color,
    pub purple: Color,
    pub selected_bg: Color,
    pub status_bg: Color,
    pub search_bg: Color,
    pub header_bg: Color,
    pub dim: Color,
}

pub fn all_themes() -> Vec<Theme> {
    vec![
        // ─── Catppuccin Mocha ────────────────────────────────────────────────
        Theme {
            name: "catppuccin-mocha",
            bg: Color::Rgb(30, 30, 46),
            fg: Color::Rgb(205, 214, 244),
            border: Color::Rgb(88, 91, 112),
            accent: Color::Rgb(203, 166, 247),
            green: Color::Rgb(166, 227, 161),
            red: Color::Rgb(243, 139, 168),
            yellow: Color::Rgb(249, 226, 175),
            blue: Color::Rgb(137, 180, 250),
            purple: Color::Rgb(203, 166, 247),
            selected_bg: Color::Rgb(49, 50, 68),
            status_bg: Color::Rgb(24, 24, 37),
            search_bg: Color::Rgb(49, 50, 68),
            header_bg: Color::Rgb(24, 24, 37),
            dim: Color::Rgb(108, 112, 134),
        },
        // ─── Catppuccin Latte ────────────────────────────────────────────────
        Theme {
            name: "catppuccin-latte",
            bg: Color::Rgb(239, 241, 245),
            fg: Color::Rgb(76, 79, 105),
            border: Color::Rgb(172, 176, 190),
            accent: Color::Rgb(136, 57, 239),
            green: Color::Rgb(64, 160, 43),
            red: Color::Rgb(210, 15, 57),
            yellow: Color::Rgb(223, 142, 29),
            blue: Color::Rgb(30, 102, 245),
            purple: Color::Rgb(136, 57, 239),
            selected_bg: Color::Rgb(220, 224, 232),
            status_bg: Color::Rgb(204, 208, 218),
            search_bg: Color::Rgb(220, 224, 232),
            header_bg: Color::Rgb(204, 208, 218),
            dim: Color::Rgb(156, 160, 176),
        },
        // ─── Tokyo Night ─────────────────────────────────────────────────────
        Theme {
            name: "tokyo-night",
            bg: Color::Rgb(26, 27, 38),
            fg: Color::Rgb(169, 177, 214),
            border: Color::Rgb(41, 46, 66),
            accent: Color::Rgb(187, 154, 247),
            green: Color::Rgb(158, 206, 106),
            red: Color::Rgb(247, 118, 142),
            yellow: Color::Rgb(224, 175, 104),
            blue: Color::Rgb(122, 162, 247),
            purple: Color::Rgb(187, 154, 247),
            selected_bg: Color::Rgb(41, 46, 66),
            status_bg: Color::Rgb(22, 22, 30),
            search_bg: Color::Rgb(41, 46, 66),
            header_bg: Color::Rgb(22, 22, 30),
            dim: Color::Rgb(86, 95, 137),
        },
        // ─── Gruvbox Dark ────────────────────────────────────────────────────
        Theme {
            name: "gruvbox-dark",
            bg: Color::Rgb(40, 40, 40),
            fg: Color::Rgb(235, 219, 178),
            border: Color::Rgb(80, 73, 69),
            accent: Color::Rgb(250, 189, 47),
            green: Color::Rgb(184, 187, 38),
            red: Color::Rgb(251, 73, 52),
            yellow: Color::Rgb(250, 189, 47),
            blue: Color::Rgb(131, 165, 152),
            purple: Color::Rgb(211, 134, 155),
            selected_bg: Color::Rgb(60, 56, 54),
            status_bg: Color::Rgb(29, 32, 33),
            search_bg: Color::Rgb(60, 56, 54),
            header_bg: Color::Rgb(29, 32, 33),
            dim: Color::Rgb(124, 111, 100),
        },
        // ─── Nord ────────────────────────────────────────────────────────────
        Theme {
            name: "nord",
            bg: Color::Rgb(46, 52, 64),
            fg: Color::Rgb(236, 239, 244),
            border: Color::Rgb(67, 76, 94),
            accent: Color::Rgb(136, 192, 208),
            green: Color::Rgb(163, 190, 140),
            red: Color::Rgb(191, 97, 106),
            yellow: Color::Rgb(235, 203, 139),
            blue: Color::Rgb(129, 161, 193),
            purple: Color::Rgb(180, 142, 173),
            selected_bg: Color::Rgb(59, 66, 82),
            status_bg: Color::Rgb(36, 41, 51),
            search_bg: Color::Rgb(59, 66, 82),
            header_bg: Color::Rgb(36, 41, 51),
            dim: Color::Rgb(76, 86, 106),
        },
        // ─── Dracula ─────────────────────────────────────────────────────────
        Theme {
            name: "dracula",
            bg: Color::Rgb(40, 42, 54),
            fg: Color::Rgb(248, 248, 242),
            border: Color::Rgb(68, 71, 90),
            accent: Color::Rgb(189, 147, 249),
            green: Color::Rgb(80, 250, 123),
            red: Color::Rgb(255, 85, 85),
            yellow: Color::Rgb(241, 250, 140),
            blue: Color::Rgb(139, 233, 253),
            purple: Color::Rgb(189, 147, 249),
            selected_bg: Color::Rgb(68, 71, 90),
            status_bg: Color::Rgb(33, 34, 44),
            search_bg: Color::Rgb(68, 71, 90),
            header_bg: Color::Rgb(33, 34, 44),
            dim: Color::Rgb(98, 114, 164),
        },
    ]
}

pub fn theme_by_name(name: &str) -> usize {
    all_themes()
        .iter()
        .position(|t| t.name == name)
        .unwrap_or(0)
}
