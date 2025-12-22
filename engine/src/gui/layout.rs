/// Layout system for flexible, dynamic UI positioning and sizing.
/// Supports percentage-based and fixed sizing, with alignment options.

use glam::Vec2;

/// How a component should size itself relative to its parent
#[derive(Clone, Copy, Debug)]
pub enum SizeSpec {
    /// Fixed size in pixels
    Fixed(f32),
    /// Percentage of parent size (0.0 to 1.0)
    Percent(f32),
}

impl SizeSpec {
    pub fn compute(&self, parent_size: f32) -> f32 {
        match self {
            SizeSpec::Fixed(px) => *px,
            SizeSpec::Percent(pct) => parent_size * pct.clamp(0.0, 1.0),
        }
    }
}

/// Horizontal alignment
#[derive(Clone, Copy, Debug)]
pub enum HAlign {
    Left,
    Center,
    Right,
}

/// Vertical alignment
#[derive(Clone, Copy, Debug)]
pub enum VAlign {
    Top,
    Middle,
    Bottom,
}

/// Complete layout specification for a component
#[derive(Clone, Copy, Debug)]
pub struct LayoutSpec {
    pub width: SizeSpec,
    pub height: SizeSpec,
    pub h_align: HAlign,
    pub v_align: VAlign,
    pub padding: f32,
    pub margin: f32,
}

impl LayoutSpec {
    pub fn new(width: SizeSpec, height: SizeSpec) -> Self {
        LayoutSpec {
            width,
            height,
            h_align: HAlign::Center,
            v_align: VAlign::Middle,
            padding: 0.0,
            margin: 0.0,
        }
    }

    pub fn with_h_align(mut self, h_align: HAlign) -> Self {
        self.h_align = h_align;
        self
    }

    pub fn with_v_align(mut self, v_align: VAlign) -> Self {
        self.v_align = v_align;
        self
    }

    pub fn with_padding(mut self, padding: f32) -> Self {
        self.padding = padding;
        self
    }

    pub fn with_margin(mut self, margin: f32) -> Self {
        self.margin = margin;
        self
    }

    pub fn with_alignment(mut self, h_align: HAlign, v_align: VAlign) -> Self {
        self.h_align = h_align;
        self.v_align = v_align;
        self
    }
}

/// Computed layout result - actual position and size
#[derive(Clone, Copy, Debug)]
pub struct ComputedLayout {
    pub position: Vec2,
    pub scale: Vec2,
}

impl ComputedLayout {
    /// Compute layout for a single component within a parent bounds
    pub fn compute(
        spec: LayoutSpec,
        parent_x: f32,
        parent_y: f32,
        parent_width: f32,
        parent_height: f32,
    ) -> Self {
        let padded_width = (parent_width - (spec.padding * 2.0)).max(0.0);
        let padded_height = (parent_height - (spec.padding * 2.0)).max(0.0);

        let width = spec.width.compute(padded_width);
        let height = spec.height.compute(padded_height);

        let padded_x = parent_x + spec.padding;
        let padded_y = parent_y + spec.padding;

        // Compute X position based on horizontal alignment
        let x = match spec.h_align {
            HAlign::Left => padded_x + width / 2.0,
            HAlign::Center => padded_x + padded_width / 2.0,
            HAlign::Right => padded_x + padded_width - width / 2.0,
        };

        // Compute Y position based on vertical alignment
        let y = match spec.v_align {
            VAlign::Top => padded_y + height / 2.0,
            VAlign::Middle => padded_y + padded_height / 2.0,
            VAlign::Bottom => padded_y + padded_height - height / 2.0,
        };

        ComputedLayout {
            position: Vec2::new(x, y),
            scale: Vec2::new(width, height),
        }
    }

    /// Compute layout for multiple components in a row with margins between them
    pub fn compute_row(
        specs: &[LayoutSpec],
        parent_x: f32,
        parent_y: f32,
        parent_width: f32,
        parent_height: f32,
    ) -> Vec<ComputedLayout> {
        if specs.is_empty() {
            return Vec::new();
        }

        let num_components = specs.len() as f32;
        let mut result = Vec::with_capacity(specs.len());

        // Use the first spec's padding/margin as row-level values
        let first_spec = specs[0];
        let padded_x = parent_x + first_spec.padding;
        let padded_width = (parent_width - (first_spec.padding * 2.0)).max(0.0);
        let padded_height = (parent_height - (first_spec.padding * 2.0)).max(0.0);
        let padded_y = parent_y + first_spec.padding;

        // Account for margins between components
        let total_margin_space = first_spec.margin * (num_components - 1.0);
        let available_width = (padded_width - total_margin_space).max(0.0);

        // Pre-calculate all component widths
        let mut component_widths = Vec::new();
        for spec in specs.iter() {
            // For percentage specs, compute relative to the full available width per component
            let width = match spec.width {
                SizeSpec::Percent(pct) => {
                    // Percentage of the full padded width divided by number of components
                    (available_width / num_components) * pct.clamp(0.0, 1.0)
                },
                SizeSpec::Fixed(px) => px,
            };
            component_widths.push(width);
        }

        // Position components with proper centering and spacing
        for (i, spec) in specs.iter().enumerate() {
            let width = component_widths[i];
            let height = spec.height.compute(padded_height);

            // Position each component with margins between them
            let x = padded_x
                + (i as f32 * (width + first_spec.margin))
                + (width / 2.0);

            // Vertical alignment
            let y = match spec.v_align {
                VAlign::Top => padded_y + height / 2.0,
                VAlign::Middle => padded_y + padded_height / 2.0,
                VAlign::Bottom => padded_y + padded_height - height / 2.0,
            };

            result.push(ComputedLayout {
                position: Vec2::new(x, y),
                scale: Vec2::new(width, height),
            });
        }

        result
    }
}
