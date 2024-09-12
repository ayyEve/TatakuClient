/*
 * Beatmap List Component
 * Provides a list of grouped beatmaps
 * 
 * Parameters:
 * - filter_var: Option<String> -> What variable to use to filter maps
 * 
 * Values:
 *  - beatmap_list.groups: List<BeatmapGroup> -> The list of (filtered) groups
 * 
 * Types:
 * - BeatmapGroup:
 *      - maps: List<Beatmap> -> The list of maps in the group
 *      - number: Number -> The number id for this set
 * 
 * Behaviors:
 *  - beatmap_list.set_beatmap(hash: String) -> Set the current beatmap to the provided hash
 *  - beatmap_list.set_set(set_num: Number) -> Set the current set to the provided set number
 *  - beatmap_list.next_map -> Set the current beatmap to the next map in the set
 *  - beatmap_list.prev_map -> Set the current beatmap to the previous map in the set
 * 
 *  - beatmap_list.next_set -> Set current set to the next set in the list
 *  - beatmap_list.prev_set -> Set current set to the previous set in the list
 */
