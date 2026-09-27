//! サーバーの最小の状態。まだ `edaberu_core` に無いので、いったんここに置く。
//! 3周目で `edaberu_core::state` へ移す。

use twilight_model::id::{
    Id,
    marker::{ChannelMarker, GuildMarker, UserMarker},
};

/// 起動時の設定から決まる、読み書きしない値。
#[allow(clippy::struct_field_names)]
pub struct Config {
    /// 使うサーバーのID。
    pub guild_id: Id<GuildMarker>,
    /// 読むテキストチャンネルのID。
    pub text_channel_id: Id<ChannelMarker>,
    /// 対象のVCのID。
    pub voice_channel_id: Id<ChannelMarker>,
}

/// gatewayが受けた出来事で変わる、最小の状態。
#[derive(Default)]
pub struct State {
    /// 対象のVCに、`GUILD_CREATE` の時点でいたユーザーID（人かbotかの判定に使う）。
    pub voice_channel_members: Vec<Id<UserMarker>>,
}
