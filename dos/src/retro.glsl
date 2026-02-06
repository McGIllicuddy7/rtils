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
uniform vec4 pallete[256];
// NOTE: Add your custom variables here

vec3 nearest_rgb_to_pallet(vec3 col) {
  vec3 nearest = pallete[0].rgb;
  float min_delta = dot(col, nearest);
  min_delta *= min_delta;
  for (int i = 1; i < 256; i++) {
    float delta = dot(col, pallete[i].rgb);
    delta *= delta;
    if (delta < min_delta) {
      min_delta = delta;
      nearest = pallete[i].rgb;
    }
  }
  return nearest;
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
  vec4 col = vec4(0.0, 0.0, 0.0, 1.0);
  col.rgb = nearest_rgb_to_pallet(base.rgb);
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