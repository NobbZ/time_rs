// SPDX-FileCopyrightText: 2025 Norbert Melzer <timmelzer@gmail.com>
//
// SPDX-License-Identifier: MIT

//! Example demonstrating multiple aggregates with different events

use serde::{Deserialize, Serialize};
use time_rs_sourcing::{Aggregate, Command, EventHandler, EventStore, GixEventStore, StoredEvent};

// ==================== User Aggregate ====================

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
enum UserEvent {
    UserRegistered { user_id: String, email: String },
    EmailChanged { user_id: String, new_email: String },
    UserDeactivated { user_id: String },
}

impl time_rs_sourcing::message::Message for UserEvent {
    fn name(&self) -> &'static str {
        "UserEvent"
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct RegisterUser {
    user_id: String,
    email: String,
}

impl time_rs_sourcing::message::Message for RegisterUser {
    fn name(&self) -> &'static str {
        "RegisterUser"
    }
}

impl Command for RegisterUser {
    type Event = UserEvent;

    fn execute(&self) -> time_rs_sourcing::Result<Vec<Self::Event>> {
        if self.email.is_empty() {
            return Err(time_rs_sourcing::EventSourcingError::RepositoryError(
                "Email cannot be empty".to_string(),
            ));
        }

        Ok(vec![UserEvent::UserRegistered {
            user_id: self.user_id.clone(),
            email: self.email.clone(),
        }])
    }
}

#[derive(Debug, Default)]
#[allow(clippy::struct_field_names)]
struct User {
    user_id: Option<String>,
    email: Option<String>,
    is_active: bool,
}

impl Aggregate for User {
    type Event = UserEvent;

    fn apply(&mut self, event: &Self::Event) -> time_rs_sourcing::Result<()> {
        match event {
            UserEvent::UserRegistered { user_id, email } => {
                self.user_id = Some(user_id.clone());
                self.email = Some(email.clone());
                self.is_active = true;
            }
            UserEvent::EmailChanged { new_email, .. } => {
                self.email = Some(new_email.clone());
            }
            UserEvent::UserDeactivated { .. } => {
                self.is_active = false;
            }
        }
        Ok(())
    }
}

// ==================== Order Aggregate ====================

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
enum OrderEvent {
    OrderCreated {
        order_id: String,
        user_id: String,
        total: i64,
    },
    OrderPaid {
        order_id: String,
    },
    OrderShipped {
        order_id: String,
        tracking_number: String,
    },
    OrderCancelled {
        order_id: String,
    },
}

impl time_rs_sourcing::message::Message for OrderEvent {
    fn name(&self) -> &'static str {
        "OrderEvent"
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct CreateOrder {
    order_id: String,
    user_id: String,
    total: i64,
}

impl time_rs_sourcing::message::Message for CreateOrder {
    fn name(&self) -> &'static str {
        "CreateOrder"
    }
}

impl Command for CreateOrder {
    type Event = OrderEvent;

    fn execute(&self) -> time_rs_sourcing::Result<Vec<Self::Event>> {
        if self.total <= 0 {
            return Err(time_rs_sourcing::EventSourcingError::RepositoryError(
                "Order total must be positive".to_string(),
            ));
        }

        Ok(vec![OrderEvent::OrderCreated {
            order_id: self.order_id.clone(),
            user_id: self.user_id.clone(),
            total: self.total,
        }])
    }
}

#[derive(Debug, Default)]
#[allow(clippy::struct_field_names)]
struct Order {
    order_id: Option<String>,
    user_id: Option<String>,
    total: i64,
    is_paid: bool,
    is_shipped: bool,
    is_cancelled: bool,
    tracking_number: Option<String>,
}

impl Aggregate for Order {
    type Event = OrderEvent;

    fn apply(&mut self, event: &Self::Event) -> time_rs_sourcing::Result<()> {
        match event {
            OrderEvent::OrderCreated {
                order_id,
                user_id,
                total,
            } => {
                self.order_id = Some(order_id.clone());
                self.user_id = Some(user_id.clone());
                self.total = *total;
            }
            OrderEvent::OrderPaid { .. } => {
                self.is_paid = true;
            }
            OrderEvent::OrderShipped {
                tracking_number, ..
            } => {
                self.is_shipped = true;
                self.tracking_number = Some(tracking_number.clone());
            }
            OrderEvent::OrderCancelled { .. } => {
                self.is_cancelled = true;
            }
        }
        Ok(())
    }
}

// ==================== Read Model / Event Handler ====================

use std::collections::HashMap;

struct UserOrderStatsReadModel {
    user_order_counts: HashMap<String, usize>,
    user_total_spent: HashMap<String, i64>,
}

impl UserOrderStatsReadModel {
    fn new() -> Self {
        Self {
            user_order_counts: HashMap::new(),
            user_total_spent: HashMap::new(),
        }
    }

    fn get_order_count(&self, user_id: &str) -> usize {
        *self.user_order_counts.get(user_id).unwrap_or(&0)
    }

    fn get_total_spent(&self, user_id: &str) -> i64 {
        *self.user_total_spent.get(user_id).unwrap_or(&0)
    }
}

impl EventHandler for UserOrderStatsReadModel {
    type Event = OrderEvent;

    fn handle(&mut self, event: &StoredEvent<Self::Event>) -> time_rs_sourcing::Result<()> {
        match &event.event {
            OrderEvent::OrderCreated {
                user_id, total, ..
            } => {
                *self.user_order_counts.entry(user_id.clone()).or_insert(0) += 1;
                *self.user_total_spent.entry(user_id.clone()).or_insert(0) += total;
            }
            OrderEvent::OrderCancelled { order_id } => {
                // In a real system, we'd need to track which order belongs to which user
                // and adjust the stats accordingly
                println!("Order {order_id} cancelled (stats update would happen here)");
            }
            _ => {}
        }
        Ok(())
    }
}

fn main() -> time_rs_sourcing::Result<()> {
    let temp_dir = std::env::temp_dir().join("multi_aggregate_example");
    let _ = std::fs::remove_dir_all(&temp_dir);
    std::fs::create_dir_all(&temp_dir)?;

    println!("Multi-Aggregate Event Sourcing Example");
    println!("========================================\n");

    // Initialize event stores for different aggregates
    let mut user_store = GixEventStore::init(temp_dir.join("users"))?;
    let mut order_store = GixEventStore::init(temp_dir.join("orders"))?;

    // ===== User Workflow =====
    println!("User Workflow:");
    println!("--------------");

    // Execute user commands
    let register_cmd = RegisterUser {
        user_id: "user-123".to_string(),
        email: "alice@example.com".to_string(),
    };

    let user_events = register_cmd.execute()?;
    for event in &user_events {
        let commit_id = user_store.append(event)?;
        println!("User event stored: {:?} (commit: {})", event, &commit_id[..8]);
    }

    // Change email
    let email_change_event = UserEvent::EmailChanged {
        user_id: "user-123".to_string(),
        new_email: "alice.new@example.com".to_string(),
    };
    user_store.append(&email_change_event)?;
    println!("Email changed for user-123");

    // Rebuild user aggregate
    let stored_user_events = user_store.read_all::<UserEvent>()?;
    let user = User::replay(stored_user_events.iter().map(|e| e.event.clone()))?;
    println!("\nReconstructed User State:");
    println!("  User ID: {:?}", user.user_id);
    println!("  Email: {:?}", user.email);
    println!("  Active: {}", user.is_active);

    // ===== Order Workflow =====
    println!("\n\nOrder Workflow:");
    println!("---------------");

    // Execute order commands
    let create_order_cmd = CreateOrder {
        order_id: "order-456".to_string(),
        user_id: "user-123".to_string(),
        total: 99_99,
    };

    let order_events = create_order_cmd.execute()?;
    for event in &order_events {
        let commit_id = order_store.append(event)?;
        println!("Order event stored: {:?} (commit: {})", event, &commit_id[..8]);
    }

    // Add more order events
    order_store.append(&OrderEvent::OrderPaid {
        order_id: "order-456".to_string(),
    })?;
    println!("Order paid");

    order_store.append(&OrderEvent::OrderShipped {
        order_id: "order-456".to_string(),
        tracking_number: "TRACK123".to_string(),
    })?;
    println!("Order shipped");

    // Create another order
    let create_order_cmd2 = CreateOrder {
        order_id: "order-789".to_string(),
        user_id: "user-123".to_string(),
        total: 49_99,
    };
    for event in create_order_cmd2.execute()? {
        order_store.append(&event)?;
    }

    // Rebuild order aggregate for first order
    let stored_order_events = order_store.read_all::<OrderEvent>()?;
    let order = Order::replay(
        stored_order_events
            .iter()
            .filter(|e| match &e.event {
                OrderEvent::OrderCreated { order_id, .. }
                | OrderEvent::OrderPaid { order_id, .. }
                | OrderEvent::OrderShipped { order_id, .. }
                | OrderEvent::OrderCancelled { order_id } => order_id == "order-456",
            })
            .map(|e| e.event.clone()),
    )?;

    println!("\nReconstructed Order State (order-456):");
    println!("  Order ID: {:?}", order.order_id);
    println!("  User ID: {:?}", order.user_id);
    {
        #[allow(clippy::cast_precision_loss)]
        let total = order.total as f64 / 100.0;
        println!("  Total: ${total:.2}");
    }
    println!("  Paid: {}", order.is_paid);
    println!("  Shipped: {}", order.is_shipped);
    println!("  Tracking: {:?}", order.tracking_number);

    // ===== Event Handler / Read Model =====
    println!("\n\nRead Model (User Order Statistics):");
    println!("------------------------------------");

    let mut stats_handler = UserOrderStatsReadModel::new();
    stats_handler.handle_many(&stored_order_events)?;

    println!(
        "User user-123 has placed {} orders",
        stats_handler.get_order_count("user-123")
    );
    {
        #[allow(clippy::cast_precision_loss)]
        let total_spent = stats_handler.get_total_spent("user-123") as f64 / 100.0;
        println!("User user-123 has spent ${total_spent:.2}");
    }

    // Clean up
    std::fs::remove_dir_all(&temp_dir)?;

    println!("\n✓ Multi-aggregate example completed successfully!");

    Ok(())
}
