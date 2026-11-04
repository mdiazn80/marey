import * as THREE from "three";
import type { Hand } from "@/types/generated/Hand";
import { SCENE_COLORS, WORLD_UNITS_PER_MM } from "./theme";

const MAX_HANDS = 4;
// Four bones per digit, the arm, the wrist-to-palm link, the thumb's link to the wrist (its
// metacarpal has zero length), and three links across the knuckles.
const BONES_PER_HAND = 5 * 4 + 6;
// Two joints per bone, plus the palm centre.
const JOINTS_PER_HAND = BONES_PER_HAND * 2 + 1;

export const MAX_BONES = MAX_HANDS * BONES_PER_HAND;
export const MAX_JOINTS = MAX_HANDS * JOINTS_PER_HAND;

/** Width of the links drawn to give the palm a visible outline, in millimetres. */
const LINK_WIDTH_MM = 10;

const LEFT_COLOR = new THREE.Color(SCENE_COLORS.leftHand);
const RIGHT_COLOR = new THREE.Color(SCENE_COLORS.rightHand);
const UP = new THREE.Vector3(0, 1, 0);
const NO_ROTATION = new THREE.Quaternion();

/**
 * Writes hands into a pair of instanced meshes. It runs every render frame, so it reuses its
 * vectors instead of allocating new ones.
 */
export class HandMeshWriter {
  private readonly matrix = new THREE.Matrix4();
  private readonly start = new THREE.Vector3();
  private readonly end = new THREE.Vector3();
  private readonly middle = new THREE.Vector3();
  private readonly direction = new THREE.Vector3();
  private readonly rotation = new THREE.Quaternion();
  private readonly size = new THREE.Vector3();
  private boneCount = 0;
  private jointCount = 0;

  constructor(
    private readonly bones: THREE.InstancedMesh,
    private readonly joints: THREE.InstancedMesh,
  ) {}

  write(hands: Hand[]) {
    this.boneCount = 0;
    this.jointCount = 0;
    for (const hand of hands.slice(0, MAX_HANDS)) this.addHand(hand);

    this.bones.count = this.boneCount;
    this.joints.count = this.jointCount;
    this.bones.instanceMatrix.needsUpdate = true;
    this.joints.instanceMatrix.needsUpdate = true;
    if (this.bones.instanceColor) this.bones.instanceColor.needsUpdate = true;
    if (this.joints.instanceColor) this.joints.instanceColor.needsUpdate = true;
  }

  private addHand(hand: Hand) {
    const color = hand.side === "left" ? LEFT_COLOR : RIGHT_COLOR;
    const palm = hand.palm.position;
    this.addBone(hand.arm.prev, hand.arm.next, hand.arm.width * 0.4, color);
    this.addBone(hand.arm.next, hand.digits[0].bones[0].next, LINK_WIDTH_MM, color);
    for (let finger = 1; finger < 4; finger += 1) {
      this.addBone(
        hand.digits[finger].bones[0].next,
        hand.digits[finger + 1].bones[0].next,
        LINK_WIDTH_MM,
        color,
      );
    }
    this.addBone(hand.arm.next, palm, hand.palm.width * 0.3, color);
    for (const digit of hand.digits) {
      for (const bone of digit.bones) this.addBone(bone.prev, bone.next, bone.width, color);
    }
    this.start.set(palm[0], palm[1], palm[2]).multiplyScalar(WORLD_UNITS_PER_MM);
    this.addJoint(this.start, hand.palm.width * 0.2 * WORLD_UNITS_PER_MM, color);
  }

  private addBone(prev: number[], next: number[], width: number, color: THREE.Color) {
    this.start.set(prev[0], prev[1], prev[2]).multiplyScalar(WORLD_UNITS_PER_MM);
    this.end.set(next[0], next[1], next[2]).multiplyScalar(WORLD_UNITS_PER_MM);
    this.direction.subVectors(this.end, this.start);
    const length = this.direction.length();
    if (length > 1e-5) {
      const radius = width * 0.5 * WORLD_UNITS_PER_MM * 0.55;
      this.middle.addVectors(this.start, this.end).multiplyScalar(0.5);
      this.rotation.setFromUnitVectors(UP, this.direction.divideScalar(length));
      this.size.set(radius, length, radius);
      this.matrix.compose(this.middle, this.rotation, this.size);
      this.bones.setMatrixAt(this.boneCount, this.matrix);
      this.bones.setColorAt(this.boneCount, color);
      this.boneCount += 1;
    }
    const jointRadius = width * 0.5 * WORLD_UNITS_PER_MM * 0.7;
    this.addJoint(this.start, jointRadius, color);
    this.addJoint(this.end, jointRadius, color);
  }

  private addJoint(position: THREE.Vector3, radius: number, color: THREE.Color) {
    this.size.setScalar(radius);
    this.matrix.compose(position, NO_ROTATION, this.size);
    this.joints.setMatrixAt(this.jointCount, this.matrix);
    this.joints.setColorAt(this.jointCount, color);
    this.jointCount += 1;
  }
}
