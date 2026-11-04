import * as THREE from "three";
import { describe, expect, it } from "vitest";
import type { Bone } from "@/types/generated/Bone";
import type { Digit } from "@/types/generated/Digit";
import type { Hand } from "@/types/generated/Hand";
import { HandMeshWriter, MAX_BONES, MAX_JOINTS } from "./handMeshWriter";

const bone = (x: number, length = 10): Bone => ({
  prev: [x, 0, 0],
  next: [x, length, 0],
  width: 10,
});

const digit = (x: number, firstBoneLength = 10): Digit => ({
  bones: [bone(x, firstBoneLength), bone(x + 1), bone(x + 2), bone(x + 3)],
  extended: true,
});

const hand = (side: Hand["side"]): Hand => ({
  id: 1,
  side,
  confidence: 1,
  grabStrength: 0,
  pinchStrength: 0,
  pinchDistance: 0,
  visibleTimeUs: 0,
  palm: {
    position: [0, 100, 0],
    normal: [0, -1, 0],
    direction: [0, 0, -1],
    velocity: [0, 0, 0],
    width: 80,
  },
  // A thumb's metacarpal has zero length.
  digits: [digit(0, 0), digit(20), digit(40), digit(60), digit(80)],
  arm: { prev: [0, 0, 100], next: [0, 0, 50], width: 50 },
});

function writer() {
  const geometry = new THREE.BufferGeometry();
  const material = new THREE.MeshBasicMaterial();
  const bones = new THREE.InstancedMesh(geometry, material, MAX_BONES);
  const joints = new THREE.InstancedMesh(geometry, material, MAX_JOINTS);
  return { bones, joints, writer: new HandMeshWriter(bones, joints) };
}

describe("HandMeshWriter", () => {
  it("skips zero-length bones but keeps their joints", () => {
    const { bones, joints, writer: meshes } = writer();
    meshes.write([hand("left")]);
    // 20 digit bones, the arm and 5 links, minus the thumb's metacarpal.
    expect(bones.count).toBe(25);
    expect(joints.count).toBe(26 * 2 + 1);
  });

  it("colors each side differently", () => {
    const { bones, writer: meshes } = writer();
    meshes.write([hand("left"), hand("right")]);
    const [left, right] = [new THREE.Color(), new THREE.Color()];
    bones.getColorAt(0, left);
    bones.getColorAt(25, right);
    expect(left.equals(right)).toBe(false);
  });

  it("stays within the mesh capacity with too many hands", () => {
    const { bones, joints, writer: meshes } = writer();
    meshes.write(Array.from({ length: 6 }, () => hand("right")));
    expect(bones.count).toBeLessThanOrEqual(MAX_BONES);
    expect(joints.count).toBe(MAX_JOINTS);
  });

  it("empties the meshes when the hands are gone", () => {
    const { bones, joints, writer: meshes } = writer();
    meshes.write([hand("left")]);
    meshes.write([]);
    expect(bones.count).toBe(0);
    expect(joints.count).toBe(0);
  });
});
