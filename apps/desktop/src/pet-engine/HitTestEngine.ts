const HIT_RADIUS = 48;

export class HitTestEngine {
  constructor(private windowSize = 160) {}

  isSolidPixel(localX: number, localY: number): boolean {
    const cx = this.windowSize / 2;
    const cy = this.windowSize / 2;
    const dx = localX - cx;
    const dy = localY - cy;
    return dx * dx + dy * dy <= HIT_RADIUS * HIT_RADIUS;
  }
}
