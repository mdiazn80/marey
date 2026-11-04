import { useFrame } from "@react-three/fiber";
import { useMemo, useRef, type RefObject } from "react";
import * as THREE from "three";
import { streams } from "../../stores/streams";
import type { Bone } from "../../types/generated/Bone";
import type { Hand } from "../../types/generated/Hand";

/** World units per millimetre. */
export const SCALE = 0.01;

const MAX_HANDS = 4;
// Four bones per digit, the arm, the wrist-to-palm link, the thumb's link to the wrist (its
// metacarpal has zero length), and three links across the knuckles.
const BONES_PER_HAND = 5 * 4 + 6;
// Two joints per bone, plus the palm centre.
const JOINTS_PER_HAND = BONES_PER_HAND * 2 + 1;
/** Hide the hands if no frame has arrived for this long, in milliseconds. */
const STALE_AFTER_MS = 500;

/** Width of the links drawn to give the palm a visible outline, in millimetres. */
const LINK_WIDTH_MM = 10;

const LEFT_COLOR = new THREE.Color("#EEEAE0");
const RIGHT_COLOR = new THREE.Color("#FF5A36");
const UP = new THREE.Vector3(0, 1, 0);

/** Instanced meshes updated in place every render frame, without any React state. */
export function Hands({ renderCounterRef }: { renderCounterRef: RefObject<number> }) {
  const bones = useRef<THREE.InstancedMesh>(null);
  const joints = useRef<THREE.InstancedMesh>(null);
  const scratch = useMemo(
    () => ({
      matrix: new THREE.Matrix4(),
      start: new THREE.Vector3(),
      end: new THREE.Vector3(),
      middle: new THREE.Vector3(),
      direction: new THREE.Vector3(),
      rotation: new THREE.Quaternion(),
      size: new THREE.Vector3(),
      identity: new THREE.Quaternion(),
    }),
    [],
  );

  useFrame(() => {
    renderCounterRef.current += 1;
    const boneMesh = bones.current;
    const jointMesh = joints.current;
    if (!boneMesh || !jointMesh) return;

    const fresh = performance.now() - streams.frameTime < STALE_AFTER_MS;
    const hands = fresh ? (streams.frame?.hands ?? []) : [];
    let boneIndex = 0;
    let jointIndex = 0;

    const addJoint = (position: THREE.Vector3, radius: number, color: THREE.Color) => {
      scratch.size.setScalar(radius);
      scratch.matrix.compose(position, scratch.identity, scratch.size);
      jointMesh.setMatrixAt(jointIndex, scratch.matrix);
      jointMesh.setColorAt(jointIndex, color);
      jointIndex += 1;
    };

    const addBone = (prev: number[], next: number[], width: number, color: THREE.Color) => {
      scratch.start.set(prev[0], prev[1], prev[2]).multiplyScalar(SCALE);
      scratch.end.set(next[0], next[1], next[2]).multiplyScalar(SCALE);
      scratch.direction.subVectors(scratch.end, scratch.start);
      const length = scratch.direction.length();
      if (length > 1e-5) {
        const radius = width * 0.5 * SCALE * 0.55;
        scratch.middle.addVectors(scratch.start, scratch.end).multiplyScalar(0.5);
        scratch.rotation.setFromUnitVectors(UP, scratch.direction.divideScalar(length));
        scratch.size.set(radius, length, radius);
        scratch.matrix.compose(scratch.middle, scratch.rotation, scratch.size);
        boneMesh.setMatrixAt(boneIndex, scratch.matrix);
        boneMesh.setColorAt(boneIndex, color);
        boneIndex += 1;
      }
      addJoint(scratch.start, width * 0.5 * SCALE * 0.7, color);
      addJoint(scratch.end, width * 0.5 * SCALE * 0.7, color);
    };

    const addHand = (hand: Hand) => {
      const color = hand.side === "left" ? LEFT_COLOR : RIGHT_COLOR;
      const palm = hand.palm.position;
      addBone(hand.arm.prev, hand.arm.next, hand.arm.width * 0.4, color);
      addBone(hand.arm.next, hand.digits[0].bones[0].next, LINK_WIDTH_MM, color);
      for (let finger = 1; finger < 4; finger += 1) {
        addBone(
          hand.digits[finger].bones[0].next,
          hand.digits[finger + 1].bones[0].next,
          LINK_WIDTH_MM,
          color,
        );
      }
      addBone(hand.arm.next, palm, hand.palm.width * 0.3, color);
      hand.digits.forEach((digit) =>
        digit.bones.forEach((bone: Bone) => addBone(bone.prev, bone.next, bone.width, color)),
      );
      scratch.start.set(palm[0], palm[1], palm[2]).multiplyScalar(SCALE);
      addJoint(scratch.start, hand.palm.width * 0.2 * SCALE, color);
    };

    hands.slice(0, MAX_HANDS).forEach(addHand);

    boneMesh.count = boneIndex;
    jointMesh.count = jointIndex;
    boneMesh.instanceMatrix.needsUpdate = true;
    jointMesh.instanceMatrix.needsUpdate = true;
    if (boneMesh.instanceColor) boneMesh.instanceColor.needsUpdate = true;
    if (jointMesh.instanceColor) jointMesh.instanceColor.needsUpdate = true;
  });

  return (
    <>
      <instancedMesh
        ref={bones}
        args={[undefined, undefined, MAX_HANDS * BONES_PER_HAND]}
        frustumCulled={false}
      >
        <cylinderGeometry args={[1, 1, 1, 10]} />
        <meshStandardMaterial roughness={0.6} />
      </instancedMesh>
      <instancedMesh
        ref={joints}
        args={[undefined, undefined, MAX_HANDS * JOINTS_PER_HAND]}
        frustumCulled={false}
      >
        <sphereGeometry args={[1, 12, 10]} />
        <meshStandardMaterial roughness={0.4} />
      </instancedMesh>
    </>
  );
}
