use crate::app::App;
use crate::route::{HomeRoute, Route};
use ethers_core::types::{Block as EBlock, H256};
use ratatui::{prelude::*, widgets::*};

pub fn render<B: Backend>(f: &mut Frame<B>, app: &mut App, block: EBlock<H256>, rect: Rect) {
    let detail_block = Block::default()
        .title(format!("Block #{}", block.number.unwrap()))
        .border_style(if let Route::Home(HomeRoute::Block(_)) = app.route {
            Style::default().fg(Color::Green)
        } else {
            Style::default()
        })
        .borders(Borders::ALL)
        .border_type(BorderType::Plain);

    let [detail_rect] = *Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Ratio(1,1)].as_ref())
            .split(rect)
        else {
            return;
        };

    let lines = [
        format!("{:<13}: {}", "Block Height", block.number.unwrap()),
        format!("{:<13}: {}", "Block Hash", block.hash.unwrap()),
        format!("{:<13}: {}", "Timestamp", block.time().unwrap().to_string()),
        format!("{:<13}: {}", "Transactions ", block.transactions.len()),
        format!("{:<13}: {}", "Size", block.size.unwrap()),
        format!("{:<13}: {}", "Gas Used", block.gas_used),
        format!("{:<13}: {}", "Gas Limit", block.gas_limit),
    ];

    let lines = lines
        .iter()
        .map(|row| Line::from(Span::raw(row)))
        .collect::<Vec<_>>();

    let paragraph = Paragraph::new(lines)
        .block(detail_block.to_owned())
        .alignment(Alignment::Left)
        .wrap(Wrap { trim: true });

    f.render_widget(paragraph, detail_rect);
    f.render_widget(detail_block, rect);
}
