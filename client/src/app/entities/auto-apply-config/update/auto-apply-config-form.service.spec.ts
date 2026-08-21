import { beforeEach, describe, expect, it } from 'vitest';
import { TestBed } from '@angular/core/testing';

import { sampleWithNewData, sampleWithRequiredData } from '../auto-apply-config.test-samples';

import { AutoApplyConfigFormService } from './auto-apply-config-form.service';

describe('AutoApplyConfig Form Service', () => {
  let service: AutoApplyConfigFormService;

  beforeEach(() => {
    service = TestBed.inject(AutoApplyConfigFormService);
  });

  describe('Service methods', () => {
    describe('createAutoApplyConfigFormGroup', () => {
      it('should create a new form with FormControl', () => {
        const formGroup = service.createAutoApplyConfigFormGroup();

        expect(formGroup.controls).toEqual(
          expect.objectContaining({
            id: expect.any(Object),
            userId: expect.any(Object),
            mode: expect.any(Object),
            minScore: expect.any(Object),
            maxPerDay: expect.any(Object),
            sources: expect.any(Object),
          }),
        );
      });

      it('passing IAutoApplyConfig should create a new form with FormGroup', () => {
        const formGroup = service.createAutoApplyConfigFormGroup(sampleWithRequiredData);

        expect(formGroup.controls).toEqual(
          expect.objectContaining({
            id: expect.any(Object),
            userId: expect.any(Object),
            mode: expect.any(Object),
            minScore: expect.any(Object),
            maxPerDay: expect.any(Object),
            sources: expect.any(Object),
          }),
        );
      });
    });

    describe('getAutoApplyConfig', () => {
      it('should return NewAutoApplyConfig for default AutoApplyConfig initial value', () => {
        const formGroup = service.createAutoApplyConfigFormGroup(sampleWithNewData);

        const autoApplyConfig = service.getAutoApplyConfig(formGroup);

        expect(autoApplyConfig).toMatchObject(sampleWithNewData);
      });

      it('should return NewAutoApplyConfig for empty AutoApplyConfig initial value', () => {
        const formGroup = service.createAutoApplyConfigFormGroup();

        const autoApplyConfig = service.getAutoApplyConfig(formGroup);

        expect(autoApplyConfig).toMatchObject({});
      });

      it('should return IAutoApplyConfig', () => {
        const formGroup = service.createAutoApplyConfigFormGroup(sampleWithRequiredData);

        const autoApplyConfig = service.getAutoApplyConfig(formGroup);

        expect(autoApplyConfig).toMatchObject(sampleWithRequiredData);
      });
    });

    describe('resetForm', () => {
      it('passing IAutoApplyConfig should not enable id FormControl', () => {
        const formGroup = service.createAutoApplyConfigFormGroup();
        expect(formGroup.controls.id.disabled).toBe(true);

        service.resetForm(formGroup, sampleWithRequiredData);

        expect(formGroup.controls.id.disabled).toBe(true);
      });

      it('passing NewAutoApplyConfig should disable id FormControl', () => {
        const formGroup = service.createAutoApplyConfigFormGroup(sampleWithRequiredData);
        expect(formGroup.controls.id.disabled).toBe(true);

        service.resetForm(formGroup, { id: null });

        expect(formGroup.controls.id.disabled).toBe(true);
      });
    });
  });
});
