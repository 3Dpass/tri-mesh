//! See [Mesh](crate::mesh::Mesh).

use crate::mesh::*;
use crate::TriMeshResult;

/// # Validity
impl Mesh {
    ///
    /// WARNING: DO NOT USE IN PRODUCTION!
    ///
    /// This method tests if the mesh is valid, i.e. has correct connectivity and orientation and contains no degenerate triangles.
    /// Intended only to be used in development and unit tests.
    ///
    /// # Errors
    ///
    /// If the mesh is not valid, an [MeshError::MeshIsInvalid] error with a description of the problem is returned.
    ///
    pub fn is_valid(&self) -> TriMeshResult<()> {
        for vertex_id in self.vertex_iter() {
            if let Some(halfedge_id) = self.walker_from_vertex(vertex_id).halfedge_id() {
                if !self.halfedge_iter().any(|he_id| he_id == halfedge_id) {
                    Err(MeshError::MeshIsInvalid(format!(
                        "Vertex {} points to an invalid halfedge {}",
                        vertex_id, halfedge_id
                    )))?;
                }
                if self
                    .walker_from_vertex(vertex_id)
                    .as_twin()
                    .vertex_id()
                    .unwrap()
                    != vertex_id
                {
                    Err(MeshError::MeshIsInvalid(format!("Halfedge {} pointed to by vertex {} does not start in that vertex, but instead in {}", self.walker_from_vertex(vertex_id).halfedge_id().unwrap(), vertex_id, self.walker_from_vertex(vertex_id).as_twin().vertex_id().unwrap())))?;
                }
            } else {
                Err(MeshError::MeshIsInvalid(format!(
                    "Vertex {} does not point to a halfedge",
                    vertex_id
                )))?;
            }
        }
        for halfedge_id in self.halfedge_iter() {
            let walker = self.walker_from_halfedge(halfedge_id);

            if let Some(twin_id) = walker.twin_id() {
                if !self.halfedge_iter().any(|he_id| he_id == twin_id) {
                    Err(MeshError::MeshIsInvalid(format!(
                        "Halfedge {} points to an invalid twin halfedge {}",
                        halfedge_id, twin_id
                    )))?;
                }
                if self.walker_from_halfedge(twin_id).twin_id().unwrap() != halfedge_id {
                    Err(MeshError::MeshIsInvalid(format!(
                        "Halfedge twin pointed to by halfedge {} does not point back to halfedge",
                        halfedge_id
                    )))?;
                }
                if self.walker_from_halfedge(twin_id).vertex_id() == walker.vertex_id() {
                    Err(MeshError::MeshIsInvalid( format!("Invalid orientation: The halfedge {} and its twin halfedge {} points to the same vertex {}", halfedge_id, twin_id, walker.vertex_id().unwrap())))?;
                }
            } else {
                Err(MeshError::MeshIsInvalid(format!(
                    "Halfedge {} does not point to a twin halfedge",
                    halfedge_id
                )))?;
            }

            if let Some(vertex_id) = walker.vertex_id() {
                if !self.vertex_iter().any(|vid| vid == vertex_id) {
                    Err(MeshError::MeshIsInvalid(format!(
                        "Halfedge {} points to an invalid vertex {}",
                        halfedge_id, vertex_id
                    )))?;
                }
            } else {
                Err(MeshError::MeshIsInvalid(format!(
                    "Halfedge {} does not point to a vertex",
                    halfedge_id
                )))?;
            }

            if let Some(face_id) = walker.face_id() {
                if !self.face_iter().any(|fid| fid == face_id) {
                    Err(MeshError::MeshIsInvalid(format!(
                        "Halfedge {} points to an invalid face {}",
                        halfedge_id, face_id
                    )))?;
                }
                if walker.next_id().is_none() {
                    Err(MeshError::MeshIsInvalid(format!(
                        "Halfedge {} points to a face but not a next halfedge",
                        halfedge_id
                    )))?;
                }
            }

            if let Some(next_id) = walker.next_id() {
                if !self.halfedge_iter().any(|he_id| he_id == next_id) {
                    Err(MeshError::MeshIsInvalid(format!(
                        "Halfedge {} points to an invalid next halfedge {}",
                        halfedge_id, next_id
                    )))?;
                }
                if walker.face_id().is_none() {
                    Err(MeshError::MeshIsInvalid(format!(
                        "Halfedge {} points to a next halfedge but not a face",
                        halfedge_id
                    )))?;
                }
                if self.walker_from_halfedge(next_id).previous_id().unwrap() != halfedge_id {
                    Err(MeshError::MeshIsInvalid(format!(
                        "Halfedge next pointed to by halfedge {} does not point back to halfedge",
                        halfedge_id
                    )))?;
                }
            }

            if self.edge_length(halfedge_id) < 0.00001 {
                Err(MeshError::MeshIsInvalid(format!(
                    "Length of edge {} is too small ({})",
                    halfedge_id,
                    self.edge_length(halfedge_id)
                )))?;
            }
        }
        for face_id in self.face_iter() {
            if let Some(halfedge_id) = self.walker_from_face(face_id).halfedge_id() {
                if !self.halfedge_iter().any(|he_id| he_id == halfedge_id) {
                    Err(MeshError::MeshIsInvalid(format!(
                        "Face {} points to an invalid halfedge {}",
                        face_id, halfedge_id
                    )))?;
                }
                if self.walker_from_face(face_id).face_id().unwrap() != face_id {
                    Err(MeshError::MeshIsInvalid(format!(
                        "Halfedge pointed to by face {} does not point to back to face",
                        face_id
                    )))?;
                }
            } else {
                Err(MeshError::MeshIsInvalid(format!(
                    "Face {} does not point to a halfedge",
                    face_id
                )))?;
            }

            if self.face_area(face_id) < 0.00001 {
                Err(MeshError::MeshIsInvalid(format!(
                    "Area of face {} is too small ({})",
                    face_id,
                    self.face_area(face_id)
                )))?;
            }
        }

        for vertex_id1 in self.vertex_iter() {
            for vertex_id2 in self.vertex_iter() {
                if self.connecting_edge(vertex_id1, vertex_id2).is_some()
                    != self.connecting_edge(vertex_id2, vertex_id1).is_some()
                {
                    Err(MeshError::MeshIsInvalid(format!(
                        "Vertex {} and Vertex {} is connected one way, but not the other way",
                        vertex_id1, vertex_id2
                    )))?;
                }
                let mut found = false;
                for halfedge_id in self.vertex_halfedge_iter(vertex_id1) {
                    if self.walker_from_halfedge(halfedge_id).vertex_id().unwrap() == vertex_id2 {
                        if found {
                            Err(MeshError::MeshIsInvalid(format!(
                                "Vertex {} and Vertex {} is connected by multiple edges",
                                vertex_id1, vertex_id2
                            )))?;
                        }
                        found = true;
                    }
                }
            }
        }
        Ok(())
    }
}
