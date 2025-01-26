use crate::constants::CONTENT_AREA_WIDTH;
use crate::renderer::css::cssom::StyleSheet;
use crate::renderer::dom::api::get_target_element_node;
use crate::renderer::dom::node::ElementKind;
use crate::renderer::dom::node::Node;
use crate::renderer::layout::layout_object::create_layout_object;
use crate::renderer::layout::layout_object::LayoutObject;
use crate::renderer::layout::layout_object::LayoutObjectKind;
use crate::renderer::layout::layout_object::LayoutPoint;
use crate::renderer::layout::layout_object::LayoutSize;
use alloc::rc::Rc;
use core::cell::RefCell;

#[derive(Debug, Clone)]
pub struct LayoutView {
    root: Option<Rc<RefCell<LayoutObject>>>,
}

impl LayoutView {
    pub fn new(root: Rc<RefCell<Node>>, cssom: &StyleSheet) -> Self {
        let body_root = get_target_element_node(Some(root), ElementKind::Body);

        let mut tree = Self {
            root: build_layout_tree(&body_root, &None, cssom),
        };

        tree.update_layout();

        tree
    }

    fn calculate_node_size(node: &Option<Rc<RefCell<LayoutObject>>>, parent_size: LayoutSize) {
        if let Some(n) = node {
            // ノードがブロック要素の場合、子ノードのレイアウトを計算する前にサイズを横幅を決める
            if n.borrow_mut().kind() == LayoutObjectKind::Block {
                n.borrow_mut().compute_size(parent_size);
            };

            let first_child = n.borrow().first_child();
            Self::calculate_node_size(&first_child, n.borrow().size());

            let next_sibling = n.borrow().next_sibling();
            Self::calculate_node_size(&next_sibling, parent_size);

            // 子ノードのサイズを計算した後に、サイズを計算する
            // ブロック要素の場合、高さは子要素の高さに依存する
            // インライン要素の場合、高さも横幅も子要素に依存する
            n.borrow_mut().compute_size(parent_size);
        }
    }

    fn calculate_node_position(
        node: &Option<Rc<RefCell<LayoutObject>>>,
        parent_point: LayoutPoint,
        previous_sibling_kind: LayoutObjectKind,
        previous_sibling_point: Option<LayoutPoint>,
        previous_sibling_size: Option<LayoutSize>,
    ) {
        if let Some(n) = node {
            n.borrow_mut().compute_position(
                parent_point,
                previous_sibling_kind,
                previous_sibling_point,
                previous_sibling_size,
            );

            let first_child = n.borrow().first_child();
            Self::calculate_node_position(
                &first_child,
                n.borrow().point(),
                LayoutObjectKind::Block,
                None,
                None,
            );

            let next_sibling = n.borrow().next_sibling();
            Self::calculate_node_position(
                &next_sibling,
                parent_point,
                n.borrow().kind(),
                Some(n.borrow().point()),
                Some(n.borrow().size()),
            );
        }
    }

    fn update_layout(&mut self) {
        Self::calculate_node_size(&self.root, LayoutSize::new(CONTENT_AREA_WIDTH, 0));

        Self::calculate_node_position(
            &self.root,
            LayoutPoint::new(0, 0),
            LayoutObjectKind::Block,
            None,
            None,
        );
    }

    pub fn root(&self) -> Option<Rc<RefCell<LayoutObject>>> {
        self.root.clone()
    }
}

fn build_layout_tree(
    node: &Option<Rc<RefCell<Node>>>,
    parent_obj: &Option<Rc<RefCell<LayoutObject>>>,
    cssom: &StyleSheet,
) -> Option<Rc<RefCell<LayoutObject>>> {
    let mut target_node = node.clone();
    let mut layout_object = create_layout_object(node, parent_obj, cssom);
    while layout_object.is_none() {
        if let Some(n) = target_node {
            target_node = n.borrow().next_sibling().clone();
            layout_object = create_layout_object(&target_node, parent_obj, cssom);
        } else {
            return layout_object;
        }
    }

    if let Some(n) = target_node {
        let original_first_child = n.borrow().first_child();
        let original_next_sibling = n.borrow().next_sibling();
        let mut first_child = build_layout_tree(&original_first_child, &layout_object, cssom);
        let mut next_sibling = build_layout_tree(&original_next_sibling, &None, cssom);

        // display: none の場合、レイアウトツリーには追加しない走査は継続
        if first_child.is_none() && original_first_child.is_some() {
            let mut original_dom_node = original_first_child
                .expect("first child should exist")
                .borrow()
                .next_sibling();

            loop {
                first_child = build_layout_tree(&original_dom_node, &layout_object, cssom);

                if first_child.is_none() && original_dom_node.is_some() {
                    original_dom_node = original_dom_node
                        .expect("next sibling should exist")
                        .borrow()
                        .next_sibling();
                    continue;
                }

                break;
            }
        }

        if next_sibling.is_none() && n.borrow().next_sibling().is_some() {
            let mut original_dom_node = original_next_sibling
                .expect("next sibling should exist")
                .borrow()
                .next_sibling();

            loop {
                next_sibling = build_layout_tree(&original_dom_node, &None, cssom);

                if next_sibling.is_none() && original_dom_node.is_some() {
                    original_dom_node = original_dom_node
                        .expect("next sibling should exist")
                        .borrow()
                        .next_sibling();
                    continue;
                }

                break;
            }
        }

        let obj = match layout_object {
            Some(ref obj) => obj,
            None => panic!("render object should exist here"),
        };
        obj.borrow_mut().set_first_child(first_child);
        obj.borrow_mut().set_next_sibling(next_sibling);
    }

    layout_object
}
