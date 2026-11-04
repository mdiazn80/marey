import { useFrame } from "@react-three/fiber";
import { useRef, type RefObject } from "react";
import type * as THREE from "three";
import { currentHands } from "@/stores/streams";
import { HandMeshWriter, MAX_BONES, MAX_JOINTS } from "./handMeshWriter";

/** Instanced meshes updated in place every render frame, without any React state. */
export function Hands({ renderCounterRef }: { renderCounterRef: RefObject<number> }) {
  const bones = useRef<THREE.InstancedMesh>(null);
  const joints = useRef<THREE.InstancedMesh>(null);
  const writer = useRef<HandMeshWriter | null>(null);

  useFrame(() => {
    renderCounterRef.current += 1;
    if (!bones.current || !joints.current) return;
    writer.current ??= new HandMeshWriter(bones.current, joints.current);
    writer.current.write(currentHands());
  });

  return (
    <>
      <instancedMesh ref={bones} args={[undefined, undefined, MAX_BONES]} frustumCulled={false}>
        <cylinderGeometry args={[1, 1, 1, 10]} />
        <meshStandardMaterial roughness={0.6} />
      </instancedMesh>
      <instancedMesh ref={joints} args={[undefined, undefined, MAX_JOINTS]} frustumCulled={false}>
        <sphereGeometry args={[1, 12, 10]} />
        <meshStandardMaterial roughness={0.4} />
      </instancedMesh>
    </>
  );
}
