import { Grid, OrbitControls } from "@react-three/drei";
import { Canvas } from "@react-three/fiber";
import { useMemo, type RefObject } from "react";
import * as THREE from "three";
import { Hands, SCALE } from "./Hands";

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

function volumeEdges({ hFovDeg, vFovDeg, rangeMm }: VolumeShape): THREE.BufferGeometry {
  const range = rangeMm > 0 ? rangeMm : DEFAULT_RANGE_MM;
  const halfAngle = (degrees: number, fallback: number) =>
    (((degrees > 0 ? degrees : fallback) / 2) * Math.PI) / 180;
  const halfX = Math.min(
    Math.tan(halfAngle(hFovDeg, DEFAULT_FOV_DEG.horizontal)) * range,
    MAX_HALF_EXTENT_MM,
  );
  const halfZ = Math.min(
    Math.tan(halfAngle(vFovDeg, DEFAULT_FOV_DEG.vertical)) * range,
    MAX_HALF_EXTENT_MM,
  );
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
      points.map((value) => value * SCALE),
      3,
    ),
  );
  return geometry;
}

export function Scene({
  volume,
  renderCounterRef,
}: {
  volume: VolumeShape;
  renderCounterRef: RefObject<number>;
}) {
  const edges = useMemo(() => volumeEdges(volume), [volume]);
  return (
    <Canvas camera={{ position: [0, 4.5, 7], fov: 45 }} dpr={[1, 2]}>
      <color attach="background" args={["#0E0D0B"]} />
      <ambientLight intensity={0.6} />
      <directionalLight position={[3, 8, 5]} intensity={1.4} />
      <Grid
        args={[20, 20]}
        cellSize={0.5}
        sectionSize={2}
        cellColor="#2a2720"
        sectionColor="#454137"
        fadeDistance={22}
        infiniteGrid
      />
      <mesh position={[0, 0.06, 0]}>
        <boxGeometry args={[0.8, 0.11, 0.3]} />
        <meshStandardMaterial color="#8f8a7d" metalness={0.5} roughness={0.4} />
      </mesh>
      <lineSegments geometry={edges}>
        <lineBasicMaterial color="#FF5A36" transparent opacity={0.4} />
      </lineSegments>
      <Hands renderCounterRef={renderCounterRef} />
      <OrbitControls target={[0, 2, 0]} maxPolarAngle={Math.PI * 0.55} />
    </Canvas>
  );
}
