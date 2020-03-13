// Copyright 2018-2020 Parity Technologies (UK) Ltd.
// This file is part of Substrate.

// Substrate is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// Substrate is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with Substrate.  If not, see <http://www.gnu.org/licenses/>.

mod runcmd;
mod export_blocks_cmd;
mod build_spec_cmd;
//mod import_blocks_cmd;
//mod check_block_cmd;
//mod revert_cmd;
//mod purge_chain_cmd;

use std::fmt::Debug;
use std::path::PathBuf;
use structopt::StructOpt;
use core::future::Future;
use core::pin::Pin;
use std::sync::Arc;

use app_dirs::{AppInfo, AppDataType};
use sc_service::{
	Configuration, ChainSpecExtension, RuntimeGenesis, ServiceBuilderCommand, ChainSpec,
	config::KeystoreConfig, config::DatabaseConfig, config::NetworkConfiguration, Roles,
	PruningMode,
};
use sp_runtime::traits::{Block as BlockT, Header as HeaderT};

use crate::error;
use crate::SubstrateCLI;
use crate::CliConfiguration;
use crate::params::SharedParams;

pub use crate::commands::runcmd::RunCmd;
pub use crate::commands::build_spec_cmd::BuildSpecCmd;
pub use crate::commands::export_blocks_cmd::ExportBlocksCmd;
/*
pub use crate::commands::import_blocks_cmd::ImportBlocksCmd;
pub use crate::commands::check_block_cmd::CheckBlockCmd;
pub use crate::commands::revert_cmd::RevertCmd;
pub use crate::commands::purge_chain_cmd::PurgeChainCmd;
*/

/// default sub directory to store network config
pub(crate) const DEFAULT_NETWORK_CONFIG_PATH : &'static str = "network";

/// All core commands that are provided by default.
///
/// The core commands are split into multiple subcommands and `Run` is the default subcommand. From
/// the CLI user perspective, it is not visible that `Run` is a subcommand. So, all parameters of
/// `Run` are exported as main executable parameters.
#[derive(Debug, Clone, StructOpt)]
pub enum Subcommand {
	/// Build a spec.json file, outputing to stdout.
	BuildSpec(BuildSpecCmd),

	/// Export blocks to a file.
	ExportBlocks(ExportBlocksCmd),

	/*
	/// Import blocks from file.
	ImportBlocks(ImportBlocksCmd),

	/// Validate a single block.
	CheckBlock(CheckBlockCmd),

	/// Revert chain to the previous state.
	Revert(RevertCmd),

	/// Remove the whole chain data.
	PurgeChain(PurgeChainCmd),
	*/
}

impl Subcommand {
	/// Get the shared parameters of a `CoreParams` command
	pub fn get_shared_params(&self) -> &SharedParams {
		use Subcommand::*;

		match self {
			BuildSpec(params) => &params.shared_params,
			ExportBlocks(params) => &params.shared_params,
			/*
			ImportBlocks(params) => &params.shared_params,
			CheckBlock(params) => &params.shared_params,
			Revert(params) => &params.shared_params,
			PurgeChain(params) => &params.shared_params,
			*/
		}
	}

	/// Initialize substrate. This must be done only once.
	///
	/// This method:
	///
	/// 1. Set the panic handler
	/// 2. Raise the FD limit
	/// 3. Initialize the logger
	pub fn init<C: SubstrateCLI<G, E>, G, E>(&self) -> error::Result<()>
	where
		G: RuntimeGenesis,
		E: ChainSpecExtension,
	{
		self.get_shared_params().init::<C, G, E>()
	}
}

impl CliConfiguration for Subcommand
{
	fn get_base_path(&self) -> Option<&PathBuf> {
		self.get_shared_params().base_path.as_ref()
	}
	fn get_is_dev(&self) -> bool {
		self.get_shared_params().dev
	}
	fn get_keystore_config(&self, _base_path: &PathBuf) -> error::Result<KeystoreConfig> {
		Ok(KeystoreConfig::InMemory)
	}
	fn get_database_config(&self, base_path: &PathBuf, cache_size: Option<usize>) -> DatabaseConfig { self.get_shared_params().get_database_config(base_path, cache_size) }
	fn get_chain_spec<C: SubstrateCLI<G, E>, G, E>(&self) -> error::Result<ChainSpec<G, E>>
	where
		G: RuntimeGenesis,
		E: ChainSpecExtension,
	{ self.get_shared_params().get_chain_spec::<C, G, E>() }
	fn init<C: SubstrateCLI<G, E>, G, E>(&self) -> error::Result<()>
	where
		G: RuntimeGenesis,
		E: ChainSpecExtension,
	{ self.get_shared_params().init::<C, G, E>() }

	fn get_pruning(&self, is_dev: bool) -> error::Result<PruningMode> {
		match self {
			Subcommand::BuildSpec(_) => Ok(Default::default()),
			Subcommand::ExportBlocks(cmd) => cmd.pruning_params.get_pruning(Roles::FULL, is_dev),
		}
	}

	fn get_network_config<G, E>(
		&self,
		chain_spec: &ChainSpec<G, E>,
		is_dev: bool,
		base_path: &PathBuf,
		client_id: &str,
	) -> error::Result<NetworkConfiguration>
	where
		G: RuntimeGenesis,
		E: ChainSpecExtension,
	{
		// TODO: we shouldn't have to set things in the middle of NetworkConfiguration
		let config_path = base_path.join(crate::DEFAULT_NETWORK_CONFIG_PATH);

		match self {
			Subcommand::BuildSpec(cmd) => {
				let config = NetworkConfiguration::default();

				Ok(NetworkConfiguration {
					node_key: cmd.node_key_params.get_node_key::<G, E>(Some(&config_path))?,
					..config
				})
			},
			Subcommand::ExportBlocks(_) => Ok(Default::default()),
		}
	}
}
