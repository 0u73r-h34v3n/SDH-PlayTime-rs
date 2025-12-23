# SDH-PlayTime-rs

Rewritten python BackEnd of [PlayTime](https://github.com/0u73r-h34v3n/SDH-PlayTime) plugin to rust.

- **playtime-core**: BackEnd Core 
- **playtime-pyo3**: Python bindings via PyO3 for Decky Loader integration
- **playtime-server**: HTTP server for remote access (Work In Progress)

## `main.py` Compatibility
- [ ] `add_time`
- [ ] `daily_statistics_for_period`
- [ ] `statistics_for_last_two_weeks`
- [ ] `fetch_playtime_information`
- [ ] `per_game_overall_statistics`
- [ ] `short_per_game_overall_statistics`
- [ ] `apply_manual_time_correction`
- [ ] `get_game`
- [ ] `get_file_sha256`
- [ ] `get_games_dictionary`
- [ ] `save_game_checksum`
- [ ] `save_game_checksum_bulk`
- [ ] `remove_game_checksum`
- [ ] `remove_all_game_checksum`
- [ ] `remove_all_checksums`
- [ ] `get_games_checksum`
- [ ] `link_game_to_game_with_checksum`
- [ ] `has_data_before`
