import type { Writable } from 'svelte/store';

export interface OS {
	ios_ver: string;
	build_num: string;
}

export interface Storage {
	total: number;
	used: number;
	available: number;
}

export interface Battery {
	level: number;
	health: number;
	cycleCounts: number;
}

export interface Hardware {
	model: string;
	modelNumber: string;
	region: string;
}

export interface DeviceContextType {
	hardware: Writable<Hardware>;
	battery: Writable<Battery>;
	os: Writable<OS>;
	storage: Writable<Storage>;
	connected: Writable<boolean>;
}
