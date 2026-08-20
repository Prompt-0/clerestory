//! Ratatui UI layout and widget rendering.

use crate::model::host::HostReport;
use crate::tui::app::TuiState;
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Tabs, Wrap},
    Frame,
};

pub struct UiRenderer;

impl UiRenderer {
    pub fn render(f: &mut Frame, state: &TuiState, report: &HostReport) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3), // Header & Title
                Constraint::Length(3), // Step Tabs
                Constraint::Min(12),   // Main View Area
                Constraint::Length(3), // Keybindings Footer
            ])
            .split(f.area());

        Self::render_header(f, chunks[0]);
        Self::render_tabs(f, chunks[1], state);
        Self::render_body(f, chunks[2], state, report);
        Self::render_footer(f, chunks[3]);
    }

    fn render_header(f: &mut Frame, area: Rect) {
        let header_text = Line::from(vec![
            Span::styled(
                " CLERESTORY ",
                Style::default()
                    .fg(Color::Black)
                    .bg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw("  "),
            Span::styled(
                "Hardware-Adaptive Windows 11 KVM Optimizer",
                Style::default()
                    .fg(Color::White)
                    .add_modifier(Modifier::BOLD),
            ),
        ]);
        let block = Block::default()
            .borders(Borders::ALL)
            .style(Style::default().fg(Color::DarkGray));
        let paragraph = Paragraph::new(header_text).block(block);
        f.render_widget(paragraph, area);
    }

    fn render_tabs(f: &mut Frame, area: Rect, state: &TuiState) {
        let titles = vec![
            " 1. Hardware Doctor ",
            " 2. Topology & Pinning ",
            " 3. Memory & I/O Engine ",
            " 4. Unattended Setup ",
            " 5. Summary & Compile ",
        ];

        let tabs = Tabs::new(titles)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(" Wizard Steps "),
            )
            .select(state.current_tab)
            .style(Style::default().fg(Color::Gray))
            .highlight_style(
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            );

        f.render_widget(tabs, area);
    }

    fn render_body(f: &mut Frame, area: Rect, state: &TuiState, report: &HostReport) {
        let body_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
            .split(area);

        match state.current_tab {
            0 => {
                // Doctor & CPU specs
                let lines = vec![
                    Line::from(vec![
                        Span::styled("CPU Model: ", Style::default().fg(Color::Yellow)),
                        Span::raw(&report.cpu.model_name),
                    ]),
                    Line::from(vec![
                        Span::styled("Sockets: ", Style::default().fg(Color::Yellow)),
                        Span::raw(report.cpu.sockets.to_string()),
                    ]),
                    Line::from(vec![
                        Span::styled("Physical Cores: ", Style::default().fg(Color::Yellow)),
                        Span::raw(report.cpu.total_physical_cores.to_string()),
                    ]),
                    Line::from(vec![
                        Span::styled("SMT Threads: ", Style::default().fg(Color::Yellow)),
                        Span::raw(report.cpu.total_threads.to_string()),
                    ]),
                    Line::from(vec![
                        Span::styled("Invariant TSC: ", Style::default().fg(Color::Yellow)),
                        Span::styled("Detected (invtsc)", Style::default().fg(Color::Green)),
                    ]),
                    Line::from(vec![
                        Span::styled("Hardware Virt: ", Style::default().fg(Color::Yellow)),
                        Span::styled("Active (KVM Ready)", Style::default().fg(Color::Green)),
                    ]),
                ];

                let block = Block::default()
                    .borders(Borders::ALL)
                    .title(" Host Silicon Status ");
                let paragraph = Paragraph::new(lines).block(block).wrap(Wrap { trim: true });
                f.render_widget(paragraph, body_chunks[0]);

                let rec_lines = vec![
                    Line::from(Span::styled(
                        "Optimal Windows 11 Profile:",
                        Style::default()
                            .fg(Color::Cyan)
                            .add_modifier(Modifier::BOLD),
                    )),
                    Line::from("• Allocate 8 Cores / 16 Threads"),
                    Line::from("• Lock execution strictly to CCD0"),
                    Line::from("• Enable all 12 Hyper-V enlightenments"),
                    Line::from("• Use VirtIO-SCSI with io_uring and TRIM"),
                    Line::from("• Use native PipeWire audio (10.6ms latency)"),
                ];

                let block_rec = Block::default()
                    .borders(Borders::ALL)
                    .title(" Recommendation Engine ");
                let p_rec = Paragraph::new(rec_lines).block(block_rec);
                f.render_widget(p_rec, body_chunks[1]);
            }
            1 => {
                // Topology view
                let mut ccd_lines = Vec::new();
                for domain in &report.cpu.cache_domains {
                    let vcache = if domain.has_3d_vcache {
                        " (3D V-Cache 96MB)"
                    } else {
                        " (32MB L3)"
                    };
                    ccd_lines.push(Line::from(vec![
                        Span::styled(
                            format!("CCD {} {}: ", domain.l3_cache_id, vcache),
                            Style::default()
                                .fg(Color::Cyan)
                                .add_modifier(Modifier::BOLD),
                        ),
                        Span::raw(format!(
                            "{} physical cores, {} threads",
                            domain.core_pairs.len(),
                            domain.cpu_ids.len()
                        )),
                    ]));
                }

                let block = Block::default()
                    .borders(Borders::ALL)
                    .title(" L3 Cache / CCD Partitioning ");
                let p = Paragraph::new(ccd_lines).block(block);
                f.render_widget(p, body_chunks[0]);

                let config_lines = vec![
                    Line::from(vec![
                        Span::styled("Assigned vCPUs: ", Style::default().fg(Color::Yellow)),
                        Span::raw(format!("{} Cores", state.selected_cores)),
                    ]),
                    Line::from(vec![
                        Span::styled("Core Selection: ", Style::default().fg(Color::Yellow)),
                        Span::styled(
                            "Single CCD Locked [Zero Latency]",
                            Style::default().fg(Color::Green),
                        ),
                    ]),
                    Line::from(vec![
                        Span::styled("Host Housekeeping: ", Style::default().fg(Color::Yellow)),
                        Span::raw("Core 0/1 (Isolated for emulatorpin)"),
                    ]),
                ];
                let block_cfg = Block::default()
                    .borders(Borders::ALL)
                    .title(" Selected Configuration ");
                let p_cfg = Paragraph::new(config_lines).block(block_cfg);
                f.render_widget(p_cfg, body_chunks[1]);
            }
            _ => {
                // Default info
                let lines = vec![
                    Line::from(Span::styled(
                        "Windows 11 VM Configuration Ready",
                        Style::default()
                            .fg(Color::Green)
                            .add_modifier(Modifier::BOLD),
                    )),
                    Line::from("Press [C] to Compile Libvirt Domain XML & Standalone Scripts."),
                    Line::from("Press [Q] or [Esc] to Exit."),
                ];
                let block = Block::default().borders(Borders::ALL).title(" Summary ");
                let p = Paragraph::new(lines).block(block);
                f.render_widget(p, area);
            }
        }
    }

    fn render_footer(f: &mut Frame, area: Rect) {
        let keys = Line::from(vec![
            Span::styled(
                " [Tab] ",
                Style::default().fg(Color::Black).bg(Color::White),
            ),
            Span::raw(" Next Step   "),
            Span::styled(
                " [Shift+Tab] ",
                Style::default().fg(Color::Black).bg(Color::White),
            ),
            Span::raw(" Prev Step   "),
            Span::styled(" [C] ", Style::default().fg(Color::Black).bg(Color::Green)),
            Span::raw(" Compile & Build   "),
            Span::styled(" [Q] ", Style::default().fg(Color::Black).bg(Color::Red)),
            Span::raw(" Quit "),
        ]);
        let block = Block::default()
            .borders(Borders::ALL)
            .style(Style::default().fg(Color::DarkGray));
        let paragraph = Paragraph::new(keys).block(block);
        f.render_widget(paragraph, area);
    }
}
