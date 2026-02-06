#version 330

// Input vertex attributes (from vertex shader)
in vec2 fragTexCoord;
in vec4 fragColor;

// Input uniform values
uniform sampler2D texture0;
uniform vec4 colDiffuse;
uniform float scanline;
// Output fragment color
out vec4 finalColor;
float cbrt256 = 6.34960420787;
float div = 6.0;
// NOTE: Add your custom variables here
float post_process_color_rg(float c) {
  float x = c * 255.0;
  float outx = 0.0;
  if (x > 196) {
    outx = 255.0;
  } else if (x > 98) {
    outx = 128.0;
  }
  if (x >) {
    outx = 255.0;
  } else if (x > 98) {
    outx = 128.0;
  }
}
void main() {
  // Texel color fetching from texture sampler
  // vec4 texelColor = texture(texture0, fragTexCoord);

  // NOTE: Implement here your fragment shader code

  // final color is the color from the texture
  //    times the tint color (colDiffuse)
  //    times the fragment color (interpolated vertex color)
  // finalColor = texelColor * colDiffuse * fragColor;

  vec4 base = texture(texture0, fragTexCoord);
  vec4 col;
  col.r = post_process_color(base.r);
  col.g = post_process_color(base.g);
  col.b = post_process_color(base.b);
  col.a = base.a;
  float h = fragTexCoord.y * 480.0;
  float p = 480 - scanline * 480;
  float line_delta = abs(round(h) - h);
  if (line_delta < 0.3) {
    if (line_delta < 0.05) {
      line_delta = 0.05;
    }
    col.rgb += (1. / line_delta * sqrt(line_delta)) * 0.01;
  }
  if (abs(h - p) < 0.5) {
    float delt = abs(h - p);
    delt *= 10;
    if (delt < 0.1) {
      delt = 0.1;
    }
    col.rgb += (1. / (delt * sqrt(delt))) * 0.05;
    col.rgb *= 1.0;
  }
  finalColor = col;
}