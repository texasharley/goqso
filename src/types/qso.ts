// TypeScript types for QSO data

export interface Qso {
  id: number;
  uuid: string;
  call: string;
  qso_date: string;
  time_on: string;
  band: string;
  mode: string;
  freq?: number;
  freq_rx?: number;
  dxcc?: number;
  country?: string;
  state?: string;
  gridsquare?: string;
  cqz?: number;
  ituz?: number;
  rst_sent?: string;
  rst_rcvd?: string;
  station_callsign?: string;
  my_gridsquare?: string;
  tx_pwr?: number;
  source: string;
  created_at: string;
  updated_at: string;
}

export interface NewQso {
  call: string;
  qso_date: string;
  time_on: string;
  band: string;
  mode: string;
  freq?: number;
  gridsquare?: string;
  rst_sent?: string;
  rst_rcvd?: string;
  source?: string;
}

export interface Confirmation {
  id: number;
  qso_id: number;
  source: string;
  confirmed_at: string;
  lotw_qsl_date?: string;
  lotw_credit_granted?: string;
}
