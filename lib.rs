#![cfg_attr(not(feature = "std"), no_std, no_main)]

#[ink::contract]
mod tasks {
    use ink::prelude::string::String;
    use ink::prelude::vec::Vec;
    use ink::storage::Mapping;

    /// Represents the state of a task
    #[derive(scale::Decode, scale::Encode, PartialEq, Eq)]
    #[cfg_attr(
        feature = "std",
        derive(scale_info::TypeInfo, ink::storage::traits::StorageLayout)
    )]
    pub enum TaskState {
        Todo,
        Wip,
        Done,
    }

    /// A single task
    #[derive(scale::Decode, scale::Encode, PartialEq, Eq)]
    #[cfg_attr(
        feature = "std",
        derive(scale_info::TypeInfo, ink::storage::traits::StorageLayout)
    )]
    pub struct Task {
        title: String,
        description: String,
        state: TaskState,
    }

    /// Task storage
    #[ink(storage)]
    pub struct Tasks {
        tasks: Mapping<AccountId, Vec<Task>>,
    }

    impl Tasks {
        #[ink(constructor)]
        pub fn new() -> Self {
            Self::default()
        }

        #[ink(constructor)]
        pub fn default() -> Self {
            Self {
                tasks: Mapping::new(),
            }
        }

        /// Add a task to the storage
        #[ink(message)]
        pub fn add_task(&mut self, user: AccountId, task: Task) {
            if !self.tasks.contains(user) {
                let empty_tasks: Vec<Task> = Vec::new();
                self.tasks.insert(user, &empty_tasks);
            }

            let mut user_task = self.tasks.get(user).unwrap();
            user_task.push(task);
            self.tasks.insert(user, &user_task);
        }

        /// Remove a task from the storage
        #[ink(message)]
        pub fn remove_task(&mut self, user: AccountId, task_title: String) {
            if !self.tasks.contains(user) {
                let empty_tasks: Vec<Task> = Vec::new();
                self.tasks.insert(user, &empty_tasks);
            } else if !self.tasks.get(user).unwrap().is_empty() {
                let mut tasks = self.tasks.get(user).unwrap();
                tasks.remove(
                    tasks
                        .iter()
                        .position(|t| t.title == task_title)
                        .unwrap_or_default(),
                );
                self.tasks.insert(user, &tasks);
            }
        }

        /// Fetchs the task of a single user
        #[ink(message)]
        pub fn get_task(&self, user: AccountId) -> Vec<Task> {
            if !self.tasks.contains(user) {
                let empty_tasks: Vec<Task> = Vec::new();
                empty_tasks
            } else {
                self.tasks.get(user).unwrap()
            }
        }
    }

    impl Default for Tasks {
        fn default() -> Self {
            Self::new()
        }
    }

    /// This is how you'd write end-to-end (E2E) or integration tests for ink! contracts.
    ///
    /// When running these you need to make sure that you:
    /// - Compile the tests with the `e2e-tests` feature flag enabled (`--features e2e-tests`)
    /// - Are running a Substrate node which contains `pallet-contracts` in the background
    #[cfg(all(test, feature = "e2e-tests"))]
    mod e2e_tests {
        /// Imports all the definitions from the outer scope so we can use them here.
        use super::*;

        /// A helper function used for calling contract messages.
        use ink_e2e::build_message;

        /// The End-to-End test `Result` type.
        type E2EResult<T> = std::result::Result<T, Box<dyn std::error::Error>>;

        /// We test that we can upload and instantiate the contract using its default constructor.
        #[ink_e2e::test]
        async fn default_works(mut client: ink_e2e::Client<C, E>) -> E2EResult<()> {
            // Given
            let constructor = TasksRef::default();

            // When
            let contract_account_id = client
                .instantiate("tasks", &ink_e2e::alice(), constructor, 0, None)
                .await
                .expect("instantiate failed")
                .account_id;

            // Then
            let get = build_message::<TasksRef>(contract_account_id)
                .call(|tasks| tasks.get_task(contract_account_id));
            let get_result = client.call_dry_run(&ink_e2e::alice(), &get, 0, None).await;
            let _tasks: Vec<Task> = Vec::new();
            assert!(matches!(get_result.return_value(), _tasks));

            Ok(())
        }

        /// We test that we can read and write a value from the on-chain contract contract.
        #[ink_e2e::test]
        async fn it_works(mut client: ink_e2e::Client<C, E>) -> E2EResult<()> {
            // Given
            let constructor = TasksRef::new();
            let contract_account_id = client
                .instantiate("tasks", &ink_e2e::bob(), constructor, 0, None)
                .await
                .expect("instantiate failed")
                .account_id;

            let get = build_message::<TasksRef>(contract_account_id)
                .call(|tasks| tasks.get_task(contract_account_id));
            let get_result = client.call_dry_run(&ink_e2e::bob(), &get, 0, None).await;
            let mut _tasks: Vec<Task> = Vec::new();
            assert!(matches!(get_result.return_value(), _tasks));

            // When
            let add_task = build_message::<TasksRef>(contract_account_id).call(|tasks| {
                tasks.add_task(
                    contract_account_id,
                    Task {
                        title: "Test".to_string(),
                        description: "Test".to_string(),
                        state: TaskState::Todo,
                    },
                )
            });
            let _add_task_result = client
                .call(&ink_e2e::bob(), add_task, 0, None)
                .await
                .expect("add task failed");
            _tasks.push(Task {
                title: "Test".to_string(),
                description: "Test".to_string(),
                state: TaskState::Todo,
            });
            // Then
            let get = build_message::<TasksRef>(contract_account_id)
                .call(|tasks| tasks.get_task(contract_account_id));
            let get_result = client.call_dry_run(&ink_e2e::bob(), &get, 0, None).await;
            assert!(matches!(get_result.return_value(), _tasks));

            Ok(())
        }
    }
}
