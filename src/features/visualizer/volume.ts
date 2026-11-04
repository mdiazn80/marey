import * as THREE from "three";
import { WORLD_UNITS_PER_MM } from "./theme";

/** Used when the device does not report its field of view or range. */
const DEFAULT_FOV_DEG = { horizontal: 140, vertical: 120 };
const DEFAULT_RANGE_MM = 600;
/** Keeps very wide fields of view from drawing a volume far larger than the usable space. */
const MAX_HALF_EXTENT_MM = 500;

export interface VolumeShape {
  hFovDeg: number;
  vFovDeg: number;
  rangeMm: number;
}

/** Line segments outlining the pyramid the device can see, with its apex at the device. */
export function volumeEdges({ hFovDeg, vFovDeg, rangeMm }: VolumeShape): THREE.BufferGeometry {
  const range = rangeMm > 0 ? rangeMm : DEFAULT_RANGE_MM;
  const halfExtent = (degrees: number, fallback: number) => {
    const halfAngle = (((degrees > 0 ? degrees : fallback) / 2) * Math.PI) / 180;
    return Math.min(Math.tan(halfAngle) * range, MAX_HALF_EXTENT_MM);
  };
  const halfX = halfExtent(hFovDeg, DEFAULT_FOV_DEG.horizontal);
  const halfZ = halfExtent(vFovDeg, DEFAULT_FOV_DEG.vertical);
  const corners: [number, number][] = [
    [-halfX, -halfZ],
    [halfX, -halfZ],
    [halfX, halfZ],
    [-halfX, halfZ],
  ];
  const points: number[] = [];
  corners.forEach(([x, z], index) => {
    const [nextX, nextZ] = corners[(index + 1) % corners.length];
    points.push(0, 0, 0, x, range, z);
    points.push(x, range, z, nextX, range, nextZ);
  });
  const geometry = new THREE.BufferGeometry();
  geometry.setAttribute(
    "position",
    new THREE.Float32BufferAttribute(
      points.map((value) => value * WORLD_UNITS_PER_MM),
      3,
    ),
  );
  return geometry;
}
