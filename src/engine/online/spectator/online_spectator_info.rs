use crate::prelude::*;

#[derive(Default)]
pub struct OnlineSpectatorInfo {
    /// our user's user id
    our_id: u32,

    /// buffer for outgoing spectator frames
    pub outgoing_frames: Vec<SpectatorFrame>,

    /// when was the last spectator frame sent?
    pub last_sent_frame: Instant,

    /// list of incoming spectator frames, indexed by host_id
    pub incoming_frames: HashMap<u32, Vec<SpectatorFrame>>,

    /// list each host's spectators
    /// 
    /// note that our spectators are under our own user_id
    pub spectator_list: HashMap<u32, SpectatorList>,
    
    /// list of accepted spectator host ids
    pub spectate_pending: Vec<u32>,
}
impl OnlineSpectatorInfo {
    pub fn new(our_id: u32) -> Self {
        Self {
            our_id,
            // make sure we always have a list
            spectator_list: [(our_id, SpectatorList::default())].into_iter().collect(),
            ..Default::default()
        }
    }

    pub fn currently_spectating(&self) -> Vec<u32> {
        self.incoming_frames.keys().map(|i|*i).collect()
    }

    pub fn our_spectator_list(&mut self) -> &mut SpectatorList {
        self.spectator_list.get_mut(&self.our_id).unwrap()
    }

    pub fn add_spec(&mut self, host_id: u32, user_id: u32, username: String) {
        trace!("Adding spec {user_id} to host {host_id}");
        self.spectator_list.entry(host_id).or_default().add(SpectatingUser::new(user_id, username));
    }
    pub fn remove_spec(&mut self, host_id: u32, removed_user: u32) {
        trace!("Removing spec {removed_user} from host {host_id}");
        // if host_id is 0, remove user for all hosts
        if host_id == 0 {
            // remove from all
            for specs in self.spectator_list.values_mut() {
                specs.remove(removed_user)
            }
            return;
        }

        if let Some(specs) = self.spectator_list.get_mut(&host_id) {
            specs.remove(removed_user)
        }
    }

    pub fn add_host(&mut self, host_id: u32) {
        trace!("Adding host {host_id}");
        self.incoming_frames.insert(host_id, Vec::new());
        self.spectator_list.insert(host_id, SpectatorList::default());
        self.spectate_pending.push(host_id);
    }

    pub fn remove_host(&mut self, host_id: u32) {
        trace!("Removing host {host_id}");
        // clear any incoming frames
        self.incoming_frames.remove(&host_id);
        // clear the list of spectators we have on file for that host
        self.spectator_list.remove(&host_id);
    }
}

