pub mod block;
pub mod event_handling;
pub mod statistics;
pub mod transaction;
use crate::{
    ethers::types::{BlockWithTransactionReceipts, ERC20Token, TransactionWithReceipt},
    network::IoEvent,
    route::{ActiveBlock, Route, RouteId},
    widget::StatefulList,
};
use ethers::core::types::{NameOrAddress, Transaction, TransactionReceipt, TxHash, U64};
use ratatui::widgets::{ListState, TableState};
use statistics::Statistics;
use std::{fs::File, io::Read, sync::mpsc::Sender};

pub enum InputMode {
    Normal,
    Editing,
}

pub struct App {
    routes: Vec<Route>,
    io_tx: Option<Sender<IoEvent>>,
    pub is_loading: bool,
    pub is_toggled: bool,
    pub statistics: Statistics,
    pub latest_blocks: Option<StatefulList<BlockWithTransactionReceipts<Transaction>>>,
    pub latest_transactions: Option<StatefulList<TransactionWithReceipt>>,
    //Search
    pub input_mode: InputMode,
    pub input: String,
    /// Position of cursor in the editor area.
    pub cursor_position: usize,
    //Block Detail
    pub block_detail_list_state: ListState,
    pub transactions_table_state: TableState,
    pub withdrawals_table_state: TableState,
    //Transaction Detail
    pub transaction_detail_list_state: ListState,
    //Token Data
    pub erc20_tokens: Vec<ERC20Token>,
}

impl App {
    pub fn new(io_tx: Sender<IoEvent>) -> App {
        let erc20_tokens = File::open("./data/tokens.json").map_or(vec![], |file| {
            let mut buffer = String::new();
            let mut file = std::io::BufReader::new(file);
            if file.read_to_string(&mut buffer).is_ok() {
                let tokens: Result<Vec<ERC20Token>, serde_json::Error> =
                    serde_json::from_str(&buffer);
                tokens.map_or(vec![], |tokens| tokens)
            } else {
                vec![]
            }
        });

        App {
            routes: vec![Route::default()],
            is_loading: false,
            is_toggled: false,
            io_tx: Some(io_tx),
            statistics: Statistics::new(),
            latest_blocks: None,
            latest_transactions: None,
            input_mode: InputMode::Normal,
            input: "".to_owned(),
            cursor_position: 0,
            //Block Detail
            block_detail_list_state: ListState::default(),
            transactions_table_state: TableState::default(),
            withdrawals_table_state: TableState::default(),
            //Transaction Detail
            transaction_detail_list_state: ListState::default(),
            //Token Data
            erc20_tokens,
        }
    }

    pub fn pop_current_route(&mut self) {
        if self.routes.len() > 1 {
            self.routes.pop();
        }
    }

    pub fn get_current_route(&self) -> Route {
        self.routes
            .last()
            .map_or(Route::default(), |route| route.to_owned())
    }

    pub fn set_route(&mut self, route: Route) {
        self.routes.push(route);
    }

    pub fn change_active_block(&mut self, active_block: ActiveBlock) {
        let current_route = self.get_current_route();
        self.routes.pop();
        self.routes
            .push(Route::new(current_route.get_id(), active_block));
    }

    pub fn update_block_with_transaction_receipts(
        &mut self,
        transaction_receipts: Vec<TransactionReceipt>,
    ) {
        self.routes = self
            .routes
            .to_owned()
            .iter()
            .map(|route| match route.get_id() {
                RouteId::Block(block)
                | RouteId::TransactionsOfBlock(block)
                | RouteId::WithdrawalsOfBlock(block) => {
                    let block = if let Some(block) = block {
                        let mut receipts = transaction_receipts
                            .iter()
                            .filter(|receipt| receipt.block_number == block.block.number)
                            .map(|receipt| receipt.to_owned())
                            .collect::<Vec<_>>();

                        let mut transaction_receipts = block
                            .transaction_receipts
                            .map_or(vec![], |receipts| receipts.to_owned());

                        transaction_receipts.append(&mut receipts);
                        Some(BlockWithTransactionReceipts {
                            block: block.block.to_owned(),
                            transaction_receipts: Some(transaction_receipts),
                        })
                    } else {
                        None
                    };

                    Route::new(
                        match route.get_id() {
                            RouteId::Block(_) => RouteId::Block(block),
                            RouteId::TransactionsOfBlock(_) => RouteId::TransactionsOfBlock(block),
                            RouteId::WithdrawalsOfBlock(_) => RouteId::WithdrawalsOfBlock(block),
                            _ => {
                                panic!("never occur")
                            }
                        },
                        route.get_active_block(),
                    )
                }
                _ => route.to_owned(),
            })
            .collect::<Vec<_>>();
    }

    // Send a network event to the network thread
    pub fn dispatch(&mut self, action: IoEvent) {
        // `is_loading` will be set to false again after the async action has finished in network.rs
        self.is_loading = true;
        if let Some(io_tx) = &self.io_tx {
            if let Err(e) = io_tx.send(action) {
                self.is_loading = false;
                println!("Error from dispatch {}", e);
                // TODO: handle error
            };
        }
    }

    pub fn move_cursor_left(&mut self) {
        let cursor_moved_left = self.cursor_position.saturating_sub(1);
        self.cursor_position = self.clamp_cursor(cursor_moved_left);
    }

    pub fn move_cursor_right(&mut self) {
        let cursor_moved_right = self.cursor_position.saturating_add(1);
        self.cursor_position = self.clamp_cursor(cursor_moved_right);
    }

    pub fn enter_char(&mut self, new_char: char) {
        self.input.insert(self.cursor_position, new_char);

        self.move_cursor_right();
    }

    pub fn paste(&mut self, data: String) {
        self.input = format!("{}{}", self.input, data);
        for _ in 0..data.len() {
            self.move_cursor_right();
        }
    }

    pub fn delete_char(&mut self) {
        let is_not_cursor_leftmost = self.cursor_position != 0;
        if is_not_cursor_leftmost {
            // Method "remove" is not used on the saved text for deleting the selected char.
            // Reason: Using remove on String works on bytes instead of the chars.
            // Using remove would require special care because of char boundaries.

            let current_index = self.cursor_position;
            let from_left_to_current_index = current_index - 1;

            // Getting all characters before the selected character.
            let before_char_to_delete = self.input.chars().take(from_left_to_current_index);
            // Getting all characters after selected character.
            let after_char_to_delete = self.input.chars().skip(current_index);

            // Put all characters together except the selected one.
            // By leaving the selected one out, it is forgotten and therefore deleted.
            self.input = before_char_to_delete.chain(after_char_to_delete).collect();
            self.move_cursor_left();
        }
    }

    pub fn clamp_cursor(&self, new_cursor_pos: usize) -> usize {
        new_cursor_pos.clamp(0, self.input.len())
    }

    pub fn reset_cursor(&mut self) {
        self.cursor_position = 0;
    }

    pub fn submit_message(&mut self) -> String {
        if let Some(token) = ERC20Token::find_by_ticker(&self.erc20_tokens, &self.input) {
            self.dispatch(IoEvent::GetNameOrAddressInfo {
                name_or_address: NameOrAddress::Address(token.contract_address),
            })
        } else if let Ok(transaction_hash) = self.input.parse::<TxHash>() {
            self.dispatch(IoEvent::GetTransactionWithReceipt { transaction_hash });
        } else if let Ok(i) = self.input.parse::<u64>() {
            let number = U64::from(i);
            self.dispatch(IoEvent::GetBlock { number });
        } else if let Ok(name_or_address) = self.input.parse::<NameOrAddress>() {
            self.dispatch(IoEvent::GetNameOrAddressInfo { name_or_address })
        }

        let message = self.input.to_owned();
        self.input.clear();
        self.reset_cursor();
        message
    }
}
