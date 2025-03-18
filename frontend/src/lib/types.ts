import type { Writable } from 'svelte/store';

export interface OS {
	ios_ver: string;
	build_num: string;
}

export interface Storage {
	total_storage: number;
	used_storage: number;
	available_storage: number;
}

export interface Battery {
	battery_level: number;
	battery_health: number;
	cycle_counts: number;
}

export interface Hardware {
	model: string;
	model_number: string;
	region: string;
}

export interface DeviceContextType {
	hardware: Writable<Hardware>;
	battery: Writable<Battery>;
	os: Writable<OS>;
	storage: Writable<Storage>;
	connected: Writable<boolean>;
}
