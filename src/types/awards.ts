// TypeScript types for awards tracking

export interface DxccProgress {
  worked: number;
  confirmed: number;
  total: number;
  entities?: DxccEntityStatus[];
}

export interface DxccEntityStatus {
  entity_code: number;
  entity_name: string;
  worked: boolean;
  confirmed: boolean;
  credited: boolean;
  worked_bands: string[];
  confirmed_bands: string[];
}

export interface WasProgress {
  worked: number;
  confirmed: number;
  total: number;
  states: StateStatus[];
}

export interface StateStatus {
  abbrev: string;
  name: string;
  worked: boolean;
  confirmed: boolean;
}

export interface VuccProgress {
  worked: number;
  confirmed: number;
  target: number;
  band?: string;
  grids?: GridStatus[];
}

export interface GridStatus {
  grid: string;
  worked: boolean;
  confirmed: boolean;
}
