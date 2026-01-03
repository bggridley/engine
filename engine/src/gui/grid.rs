use anyhow::Result;
use crate::gui::{GUIComponent, LayoutSpec, ComputedLayout};
use crate::renderer::RenderContext;

/// A grid row containing multiple components
pub struct GridRow {
    pub components: Vec<Box<dyn GUIComponent>>,
    pub layout_specs: Vec<LayoutSpec>,
}

impl GridRow {
    pub fn new() -> Self {
        GridRow {
            components: Vec::new(),
            layout_specs: Vec::new(),
        }
    }

    pub fn add_component(&mut self, component: Box<dyn GUIComponent>, spec: LayoutSpec) {
        self.components.push(component);
        self.layout_specs.push(spec);
    }

    /// Apply layout constraints to all components in this row
    pub fn set_layout(&mut self, parent_x: f32, parent_y: f32, parent_width: f32, parent_height: f32) {
        if self.components.is_empty() {
            return;
        }

        let layouts = ComputedLayout::compute_row(
            &self.layout_specs,
            parent_x,
            parent_y,
            parent_width,
            parent_height,
        );

        for (component, layout) in self.components.iter_mut().zip(layouts.iter()) {
            component.transform_mut().position = layout.position;
            component.transform_mut().scale = layout.scale;
        }
    }

    pub fn get_component(&self, index: usize) -> Option<&dyn GUIComponent> {
        self.components.get(index).map(|c| c.as_ref())
    }

    pub fn get_component_mut(&mut self, index: usize) -> Option<&mut Box<dyn GUIComponent>> {
        self.components.get_mut(index)
    }

    pub fn render(&self, ctx: &RenderContext, renderer: &mut crate::renderer::Renderer) -> Result<()> {
        for component in &self.components {
            component.render(ctx, renderer)?;
        }
        Ok(())
    }

    pub fn handle_mouse_down(&mut self, x: f32, y: f32) {
        for component in &mut self.components {
            component.handle_mouse_down(x, y);
        }
    }

    pub fn handle_mouse_up(&mut self, x: f32, y: f32) {
        for component in &mut self.components {
            component.handle_mouse_up(x, y);
        }
    }

    pub fn handle_mouse_move(&mut self, x: f32, y: f32) {
        for component in &mut self.components {
            component.handle_mouse_move(x, y);
        }
    }
}

impl Default for GridRow {
    fn default() -> Self {
        Self::new()
    }
}

/// A grid layout system for organizing components in rows
pub struct Grid {
    pub rows: Vec<GridRow>,
}

impl Grid {
    pub fn new() -> Self {
        Grid {
            rows: Vec::new(),
        }
    }

    pub fn add_row(&mut self) -> usize {
        self.rows.push(GridRow::new());
        self.rows.len() - 1
    }

    pub fn get_row(&self, index: usize) -> Option<&GridRow> {
        self.rows.get(index)
    }

    pub fn get_row_mut(&mut self, index: usize) -> Option<&mut GridRow> {
        self.rows.get_mut(index)
    }

    pub fn row_count(&self) -> usize {
        self.rows.len()
    }

    /// Update layout for all rows based on bounds
    pub fn set_bounds(&mut self, x: f32, y: f32, width: f32, height: f32) {
        if self.rows.is_empty() {
            return;
        }

        let row_spacing = 3.0; // Space between rows
        let num_rows = self.rows.len();
        let total_spacing = row_spacing * (num_rows - 1) as f32;

        // Calculate each row's actual height based on its component specs
        let mut row_heights = Vec::new();
        let mut total_fixed_height = 0.0;
        let mut percent_rows = 0;

        for row in &self.rows {
            if !row.layout_specs.is_empty() {
                // Use first component's height spec (assuming homogeneous row heights)
                match row.layout_specs[0].height {
                    crate::gui::SizeSpec::Fixed(h) => {
                        row_heights.push(h);
                        total_fixed_height += h;
                    }
                    crate::gui::SizeSpec::Percent(_) => {
                        row_heights.push(0.0); // Placeholder, will compute after
                        percent_rows += 1;
                    }
                }
            }
        }

        // Distribute remaining height among percent-based rows
        if percent_rows > 0 {
            let remaining = (height - total_fixed_height - total_spacing).max(0.0);
            let per_row = remaining / percent_rows as f32;
            for h in &mut row_heights {
                if *h == 0.0 {
                    *h = per_row;
                }
            }
        }

        // Apply layout with calculated row heights and spacing
        // Position rows starting from the top (high Y) going downward
        let mut current_y = y + height; // Start at top
        for (i, (row, &row_height)) in self.rows.iter_mut().zip(row_heights.iter()).enumerate() {
            current_y -= row_height; // Move down
            row.set_layout(x, current_y, width, row_height);
            if i < num_rows - 1 {
                current_y -= row_spacing; // Add spacing between rows
            }
        }
    }

    pub fn render(&self, ctx: &RenderContext, renderer: &mut crate::renderer::Renderer) -> Result<()> {
        for row in &self.rows {
            row.render(ctx, renderer)?;
        }
        Ok(())
    }

    pub fn handle_mouse_down(&mut self, x: f32, y: f32) {
        for row in &mut self.rows {
            row.handle_mouse_down(x, y);
        }
    }

    pub fn handle_mouse_up(&mut self, x: f32, y: f32) {
        for row in &mut self.rows {
            row.handle_mouse_up(x, y);
        }
    }

    pub fn handle_mouse_move(&mut self, x: f32, y: f32) {
        for row in &mut self.rows {
            row.handle_mouse_move(x, y);
        }
    }
}

impl Default for Grid {
    fn default() -> Self {
        Self::new()
    }
}

/// Layout constraints for a component (legacy - kept for compatibility)
#[derive(Clone, Copy, Debug)]
pub struct LayoutConstraints {
    pub width: f32,
    pub height: f32,
    pub x: f32,
    pub y: f32,
    pub padding: f32,
    pub margin: f32,
}

impl LayoutConstraints {
    pub fn new(x: f32, y: f32, width: f32, height: f32) -> Self {
        LayoutConstraints {
            x,
            y,
            width,
            height,
            padding: 0.0,
            margin: 0.0,
        }
    }

    pub fn with_padding(mut self, padding: f32) -> Self {
        self.padding = padding;
        self
    }

    pub fn with_margin(mut self, margin: f32) -> Self {
        self.margin = margin;
        self
    }
}
