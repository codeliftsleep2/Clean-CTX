export class Unsafe {
  run(): string {
    const label = "café";
    try {
      return `${label}:ready`;
    } catch (error) {
      return String(error);
    }
  }
}
