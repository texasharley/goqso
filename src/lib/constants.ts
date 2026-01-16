/**
 * Shared constants used across GoQSO components
 * 
 * This file consolidates constants that were previously duplicated or scattered.
 */

/**
 * Band sort order - lower numbers sort first (lower frequency first)
 */
export const BAND_ORDER: Record<string, number> = {
  "160m": 1,
  "80m": 2,
  "60m": 3,
  "40m": 4,
  "30m": 5,
  "20m": 6,
  "17m": 7,
  "15m": 8,
  "12m": 9,
  "10m": 10,
  "6m": 11,
  "2m": 12,
  "70cm": 13,
};

/**
 * Available band options for filtering
 */
export const BAND_OPTIONS = [
  "160m", "80m", "60m", "40m", "30m", "20m", "17m", "15m", "12m", "10m", "6m", "2m", "70cm"
] as const;

/**
 * Available mode options for filtering
 */
export const MODE_OPTIONS = [
  "FT8", "FT4", "CW", "SSB", "RTTY", "PSK31", "JS8"
] as const;

/**
 * Confirmation status filter options
 */
export const CONFIRM_OPTIONS = [
  { key: "lotw", label: "LoTW Confirmed" },
  { key: "eqsl", label: "eQSL Confirmed" },
  { key: "unconfirmed", label: "Unconfirmed" },
] as const;

/**
 * US state code to full name mapping
 * Used for displaying state names in Band Activity panel
 */
export const STATE_NAMES: Record<string, string> = {
  "AL": "Alabama",
  "AK": "Alaska",
  "AZ": "Arizona",
  "AR": "Arkansas",
  "CA": "California",
  "CO": "Colorado",
  "CT": "Connecticut",
  "DE": "Delaware",
  "FL": "Florida",
  "GA": "Georgia",
  "HI": "Hawaii",
  "ID": "Idaho",
  "IL": "Illinois",
  "IN": "Indiana",
  "IA": "Iowa",
  "KS": "Kansas",
  "KY": "Kentucky",
  "LA": "Louisiana",
  "ME": "Maine",
  "MD": "Maryland",
  "MA": "Massachusetts",
  "MI": "Michigan",
  "MN": "Minnesota",
  "MS": "Mississippi",
  "MO": "Missouri",
  "MT": "Montana",
  "NE": "Nebraska",
  "NV": "Nevada",
  "NH": "New Hampshire",
  "NJ": "New Jersey",
  "NM": "New Mexico",
  "NY": "New York",
  "NC": "North Carolina",
  "ND": "North Dakota",
  "OH": "Ohio",
  "OK": "Oklahoma",
  "OR": "Oregon",
  "PA": "Pennsylvania",
  "RI": "Rhode Island",
  "SC": "South Carolina",
  "SD": "South Dakota",
  "TN": "Tennessee",
  "TX": "Texas",
  "UT": "Utah",
  "VT": "Vermont",
  "VA": "Virginia",
  "WA": "Washington",
  "WV": "West Virginia",
  "WI": "Wisconsin",
  "WY": "Wyoming",
  // DC is sometimes included
  "DC": "District of Columbia",
};

/**
 * DXCC entity ID for home country (United States)
 * Used for WAS tracking - only count states for US entities
 */
export const HOME_DXCC = 291;

/**
 * DXCC entity ID for Canada
 * Used for Canadian province tracking
 */
export const CANADA_DXCC = 1;
