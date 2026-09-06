import * as assert from "assert"
import * as Automerge from "../src/index.js"

describe("Automerge", () => {
  describe("basics", () => {
    it("should round-trip an uncompressed save", () => {
      const doc = Automerge.from({ value: "x".repeat(1000) })
      const compressed = Automerge.save(doc)
      const uncompressed = Automerge.saveNoCompress(doc)

      assert.ok(uncompressed.length > compressed.length)
      assert.deepEqual(Automerge.load(uncompressed), doc)
    })

    it("should write a save into a caller-owned buffer", () => {
      const doc = Automerge.from({ value: "x".repeat(1000) })
      const expected = Automerge.save(doc)
      const output = new Uint8Array(expected.length + 3).fill(0xa5)

      const written = Automerge.saveInto(doc, output)

      assert.equal(written, expected.length)
      assert.deepEqual(output.slice(0, written), expected)
      assert.deepEqual(output.slice(written), new Uint8Array([0xa5, 0xa5, 0xa5]))
      const undersized = new Uint8Array(expected.length - 1).fill(0x5a)
      assert.throws(() => Automerge.saveInto(doc, undersized), /output buffer/)
      assert.deepEqual(undersized, new Uint8Array(expected.length - 1).fill(0x5a))

      const changed = Automerge.change(doc, d => {
        d.value = "y".repeat(1000)
      })
      const changedExpected = Automerge.save(changed)
      const changedOutput = new Uint8Array(changedExpected.length)
      assert.equal(Automerge.saveInto(changed, changedOutput), changedExpected.length)
      assert.deepEqual(changedOutput, changedExpected)
    })

    it("should allow you to load incrementally", () => {
      let doc1 = Automerge.from<any>({ foo: "bar" })
      let doc2 = Automerge.init<any>()
      doc2 = Automerge.loadIncremental(doc2, Automerge.save(doc1))
      doc1 = Automerge.change(doc1, d => (d.foo2 = "bar2"))
      doc2 = Automerge.loadIncremental(
        doc2,
        Automerge.getBackend(doc1).saveIncremental(),
      )
      doc1 = Automerge.change(doc1, d => (d.foo = "bar2"))
      doc2 = Automerge.loadIncremental(
        doc2,
        Automerge.getBackend(doc1).saveIncremental(),
      )
      doc1 = Automerge.change(doc1, d => (d.x = "y"))
      doc2 = Automerge.loadIncremental(
        doc2,
        Automerge.getBackend(doc1).saveIncremental(),
      )
      assert.deepEqual(doc1, doc2)
    })
  })
})
