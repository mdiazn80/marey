/** Formats a number as lowercase hexadecimal, padded to `digits` when given. */
export const hex = (value: number, digits = 0) => `0x${value.toString(16).padStart(digits, "0")}`;
