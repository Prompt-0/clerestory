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
                let invtsc_span = if report.cpu.has_invtsc {
                    Span::styled("Detected (invtsc)", Style::default().fg(Color::Green))
                } else {
                    Span::styled("Not detected", Style::default().fg(Color::Yellow))
                };

                let virt_span = if report.cpu.has_svm_or_vmx {
                    Span::styled(
                        "Active (Hardware Virt Ready)",
                        Style::default().fg(Color::Green),
                    )
                } else {
                    Span::styled("Missing (Check BIOS)", Style::default().fg(Color::Red))
                };

                let kvm_span = if report.security.has_kvm_device {
                    Span::styled("Present (/dev/kvm)", Style::default().fg(Color::Green))
                } else {
                    Span::styled("Missing (/dev/kvm)", Style::default().fg(Color::Red))
                };

                let swtpm_span = if report.security.swtpm_bin.is_some() {
                    Span::styled("Present (TPM 2.0)", Style::default().fg(Color::Green))
                } else {
                    Span::styled("Missing (swtpm)", Style::default().fg(Color::Yellow))
                };

                let lines = vec![
                    Line::from(vec![
                        Span::styled("CPU Model: ", Style::default().fg(Color::Yellow)),
                        Span::raw(&report.cpu.model_name),
                    ]),
                    Line::from(vec![
                        Span::styled("Vendor / Sockets: ", Style::default().fg(Color::Yellow)),
                        Span::raw(format!(
                            "{} ({} Sockets)",
                            report.cpu.vendor, report.cpu.sockets
                        )),
                    ]),
                    Line::from(vec![
                        Span::styled("Cores / Threads: ", Style::default().fg(Color::Yellow)),
                        Span::raw(format!(
                            "{} Physical / {} Threads",
                            report.cpu.total_physical_cores, report.cpu.total_threads
                        )),
                    ]),
                    Line::from(vec![
                        Span::styled("Invariant TSC: ", Style::default().fg(Color::Yellow)),
                        invtsc_span,
                    ]),
                    Line::from(vec![
                        Span::styled("Hardware Virt: ", Style::default().fg(Color::Yellow)),
                        virt_span,
                    ]),
                    Line::from(vec![
                        Span::styled("KVM Hypervisor: ", Style::default().fg(Color::Yellow)),
                        kvm_span,
                    ]),
                    Line::from(vec![
                        Span::styled("TPM 2.0 (swtpm): ", Style::default().fg(Color::Yellow)),
                        swtpm_span,
                    ]),
                ];

                let block = Block::default()
                    .borders(Borders::ALL)
                    .title(" Host Silicon Status ");
                let paragraph = Paragraph::new(lines).block(block).wrap(Wrap { trim: true });
                f.render_widget(paragraph, body_chunks[0]);

                let mut rec_lines = vec![Line::from(Span::styled(
                    "Synthesis Directives:",
                    Style::default()
                        .fg(Color::Cyan)
                        .add_modifier(Modifier::BOLD),
                ))];

                if report.cpu.is_hybrid {
                    rec_lines.push(Line::from(
                        "• Intel Hybrid detected: Pinning strictly to P-Cores",
                    ));
                } else if report.cpu.cache_domains.len() > 1 {
                    rec_lines.push(Line::from(
                        "• Multi-CCD AMD detected: Locking execution to CCD0",
                    ));
                } else {
                    rec_lines.push(Line::from(
                        "• Standard topology: Locking cores with SMT pairs",
                    ));
                }

                rec_lines.push(Line::from("• Invariant TSC + 12 Hyper-V enlightenments"));
                if report.storage.supports_io_uring {
                    rec_lines.push(Line::from("• VirtIO-SCSI with io_uring + TRIM unmap"));
                } else {
                    rec_lines.push(Line::from("• VirtIO-SCSI with Native AIO"));
                }

                if report.audio.is_pipewire {
                    rec_lines.push(Line::from("• Native PipeWire audio backend (10.6ms)"));
                }

                let block_rec = Block::default()
                    .borders(Borders::ALL)
                    .title(" Optimizer Engine ");
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
