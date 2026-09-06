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
