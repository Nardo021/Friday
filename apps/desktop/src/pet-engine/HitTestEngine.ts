export const PET_WINDOW_SIZE = 160;
/** Matches `size-24` (96px) pet circle radius at scale 1. */
export const PET_HIT_RADIUS_BASE = 48;

export class HitTestEngine {
  private radius = PET_HIT_RADIUS_BASE;
  private windowWidth = PET_WINDOW_SIZE;
  private windowHeight = PET_WINDOW_SIZE;

  setWindowSize(width: number, height: number) {
    this.windowWidth = width;
    this.windowHeight = height;
  }

  setScale(scale: number) {
    this.radius = PET_HIT_RADIUS_BASE * Math.max(0.5, scale);
  }

  isSolidPixel(localX: number, localY: number): boolean {
    const cx = this.windowWidth / 2;
    const cy = this.windowHeight / 2;
    const dx = localX - cx;
    const dy = localY - cy;
    return dx * dx + dy * dy <= this.radius * this.radius;
  }
}
