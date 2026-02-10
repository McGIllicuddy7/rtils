#version 330

// This shader is based on the basic lighting shader
// This only supports one light, which is directional, and it (of course)
// supports shadows

// Input vertex attributes (from vertex shader)
in vec3 fragPosition;
in vec2 fragTexCoord;
// in vec4 fragColor;
in vec3 fragNormal;

// Input uniform values
uniform sampler2D texture0;
uniform vec4 colDiffuse;

// Output fragment color
out vec4 finalColor;

// Input lighting values
uniform vec3 lightDir[10];
uniform vec4 lightColor[10];
uniform vec3 light_positions[10];
uniform vec4 ambient;
uniform vec3 viewPos;

// Input shadowmapping values
uniform mat4 lightVP0; // Light source view-projection matrix
uniform sampler2D smap0;
uniform mat4 lightVP1; // Light source view-projection matrix
uniform sampler2D smap1;
uniform mat4 lightVP2; // Light source view-projection matrix
uniform sampler2D smap2;
uniform mat4 lightVP3; // Light source view-projection matrix
uniform sampler2D smap3;
uniform int light_count;

uniform int shadowMapResolution;
vec4 light_and_shadow_calculations(int idx) {
  vec4 out_col;
  vec4 texelColor = texture(texture0, fragTexCoord);
  vec3 lightDot = vec3(0.0);
  vec3 normal = normalize(fragNormal);
  vec3 viewD = normalize(viewPos - fragPosition);
  vec3 specular = vec3(0.0);
  vec3 l = -lightDir[idx];

  float NdotL = max(dot(normal, l), 0.0);
  lightDot += lightColor[idx].rgb * NdotL;
  float dist = length(fragPosition - light_positions[idx]);
  float specCo = 0.0;
  if (NdotL > 0.0) {
    specCo = pow(max(0.0, dot(viewD, reflect(-(l), normal))),
                 16.0); // 16 refers to shine
  }
  specular += specCo;
  out_col = (texelColor *
             ((colDiffuse + vec4(specular, 1.0)) * vec4(lightDot, 1.0))) /
            dist;
  if (idx >= 4) {
    return out_col;
  }

  // Shadow calculations
  vec4 fragPosLightSpace = vec4(0.0);
  if (idx == 0) {
    lightVP0 *vec4(fragPosition, 1);
  } else if (idx == 1) {
    lightVP1 *vec4(fragPosition, 1);
  } else if (idx == 2) {
    lightVP2 *vec4(fragPosition, 1);
  } else {
    lightVP2 *vec4(fragPosition, 1);
  }
  fragPosLightSpace.xyz /=
      fragPosLightSpace.w; // Perform the perspective division
  fragPosLightSpace.xyz = (fragPosLightSpace.xyz + 1.0) /
                          2.0; // Transform from [-1, 1] range to [0, 1] range
  vec2 sampleCoords = fragPosLightSpace.xy;
  float curDepth = fragPosLightSpace.z;

  // Slope-scale depth bias: depth biasing reduces "shadow acne" artifacts,
  // where dark stripes appear all over the scene The solution is adding a small
  // bias to the depth In this case, the bias is proportional to the slope of
  // the surface, relative to the light
  float bias = max(0.0002 * (1.0 - dot(normal, l)), 0.00002) + 0.00001;
  int shadowCounter = 0;
  const int numSamples = 9;

  // PCF (percentage-closer filtering) algorithm:
  // Instead of testing if just one point is closer to the current point,
  // we test the surrounding points as well
  // This blurs shadow edges, hiding aliasing artifacts
  vec2 texelSize = vec2(1.0 / float(shadowMapResolution));
  for (int x = -1; x <= 1; x++) {
    for (int y = -1; y <= 1; y++) {
      if (idx == 0) {
        float sampleDepth =
            texture(smap0, sampleCoords + texelSize * vec2(x, y)).r;
        if (curDepth - bias > sampleDepth)
          shadowCounter++;
      } else if (idx == 1) {
        float sampleDepth =
            texture(smap1, sampleCoords + texelSize * vec2(x, y)).r;
        if (curDepth - bias > sampleDepth)
          shadowCounter++;
      } else if (idx == 2) {
        float sampleDepth =
            texture(smap2, sampleCoords + texelSize * vec2(x, y)).r;
        if (curDepth - bias > sampleDepth)
          shadowCounter++;
      } else {
        float sampleDepth =
            texture(smap3, sampleCoords + texelSize * vec2(x, y)).r;
        if (curDepth - bias > sampleDepth)
          shadowCounter++;
      }
    }
  }
  out_col =
      mix(out_col, vec4(0, 0, 0, 1), float(shadowCounter) / float(numSamples));
  return out_col;
}
void main() {

  // Texel color fetching from texture sampler
  vec4 texelColor = texture(texture0, fragTexCoord);

  // Add ambient lighting whether in shadow or not
  int lc = light_count;
  if (lc >= 10) {
    lc = 9;
  }
  vec4 col = vec4(0.0, 0.0, 0.0, 1.0);
  for (int i = 0; i < lc; i++) {
    col += light_and_shadow_calculations(i);
  }
  col /= float(lc);
  finalColor = col;
  finalColor += texelColor * (ambient / 10.0) * colDiffuse;
  // Gamma correction
  finalColor = pow(finalColor, vec4(1.0 / 2.2));
}
