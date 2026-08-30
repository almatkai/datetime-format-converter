/// <reference types="@raycast/api">

/* 🚧 🚧 🚧
 * This file is auto-generated from the extension's manifest.
 * Do not modify manually. Instead, update the `package.json` file.
 * 🚧 🚧 🚧 */

/* eslint-disable @typescript-eslint/ban-types */

type ExtensionPreferences = {
  /** Select Preferred Date Time Formats - 2024-11-01 */
  "isoDate": boolean,
  /** undefined - 2024-11-01 01:56:27 */
  "isoDateTime": boolean,
  /** undefined - 2024-11-01 01:56:27.000 */
  "formatWithMillis": boolean,
  /** undefined - 2024-11-01 01:56:27+05:30 */
  "formatWithTimezone": boolean,
  /** undefined - 2024-11-01T01:56:27+05:30 */
  "isoFormat": boolean,
  /** undefined - 2024-10-31T20:26:27Z */
  "utcIsoFormat": boolean,
  /** undefined - 1730406387 */
  "unixTimestamp": boolean,
  /** undefined - 1730406387000 */
  "unixMillis": boolean,
  /** undefined - 11/01/2024 */
  "localizedShortDate": boolean,
  /** undefined - 11/01/2024 1:56 AM */
  "localizedShortDateTime": boolean,
  /** undefined - November 1, 2024 1:56 AM */
  "localizedFullDateTime": boolean,
  /** undefined - Friday, November 1, 2024 1:56 AM */
  "localizedLongDateTime": boolean,
  /** undefined - 1:56 AM‎ */
  "localizedTime": boolean,
  /** undefined - 1:56:27 AM */
  "localizedSecondsTime": boolean,
  /** Default Natural Language Date Format - Default date format string to use when converting natural language dates. */
  "naturalLanguageDateFormat": string
}

/** Preferences accessible in all the extension's commands */
declare type Preferences = ExtensionPreferences

declare namespace Preferences {
  /** Preferences accessible in the `index` command */
  export type Index = ExtensionPreferences & {
  /** Auto Paste - Automatically paste clipboard content to search bar when the command is executed */
  "autoPasteClipboard": boolean
}
  /** Preferences accessible in the `insert-natural-language-date` command */
  export type InsertNaturalLanguageDate = ExtensionPreferences & {}
}

declare namespace Arguments {
  /** Arguments passed to the `index` command */
  export type Index = {}
  /** Arguments passed to the `insert-natural-language-date` command */
  export type InsertNaturalLanguageDate = {
  /** Date Prompt */
  "prompt": string,
  /** Date Format */
  "format": string
}
}

