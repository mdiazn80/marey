import { Grid, OrbitControls } from "@react-three/drei";
import { Canvas } from "@react-three/fiber";
import { useMemo, type RefObject } from "react";
import { Hands } from "./Hands";
import { SCENE_COLORS } from "./theme";
import { volumeEdges, type VolumeShape } from "./volume";

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
      <color attach="background" args={[SCENE_COLORS.background]} />
      <ambientLight intensity={0.6} />
      <directionalLight position={[3, 8, 5]} intensity={1.4} />
      <Grid
        args={[20, 20]}
        cellSize={0.5}
        sectionSize={2}
        cellColor={SCENE_COLORS.gridCell}
        sectionColor={SCENE_COLORS.gridSection}
        fadeDistance={22}
        infiniteGrid
      />
      <mesh position={[0, 0.06, 0]}>
        <boxGeometry args={[0.8, 0.11, 0.3]} />
        <meshStandardMaterial color={SCENE_COLORS.device} metalness={0.5} roughness={0.4} />
      </mesh>
      <lineSegments geometry={edges}>
        <lineBasicMaterial color={SCENE_COLORS.volume} transparent opacity={0.4} />
      </lineSegments>
      <Hands renderCounterRef={renderCounterRef} />
      <OrbitControls target={[0, 2, 0]} maxPolarAngle={Math.PI * 0.55} />
    </Canvas>
  );
}
