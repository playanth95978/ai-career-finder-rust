import { beforeEach, describe, expect, it } from 'vitest';
import { TestBed } from '@angular/core/testing';

import { sampleWithNewData, sampleWithRequiredData } from '../cv-resume.test-samples';

import { CvResumeFormService } from './cv-resume-form.service';

describe('CvResume Form Service', () => {
  let service: CvResumeFormService;

  beforeEach(() => {
    service = TestBed.inject(CvResumeFormService);
  });

  describe('Service methods', () => {
    describe('createCvResumeFormGroup', () => {
      it('should create a new form with FormControl', () => {
        const formGroup = service.createCvResumeFormGroup();

        expect(formGroup.controls).toEqual(
          expect.objectContaining({
            id: expect.any(Object),
            userId: expect.any(Object),
            title: expect.any(Object),
            template: expect.any(Object),
            data: expect.any(Object),
            versionNumber: expect.any(Object),
            createdAt: expect.any(Object),
            updatedAt: expect.any(Object),
          }),
        );
      });

      it('passing ICvResume should create a new form with FormGroup', () => {
        const formGroup = service.createCvResumeFormGroup(sampleWithRequiredData);

        expect(formGroup.controls).toEqual(
          expect.objectContaining({
            id: expect.any(Object),
            userId: expect.any(Object),
            title: expect.any(Object),
            template: expect.any(Object),
            data: expect.any(Object),
            versionNumber: expect.any(Object),
            createdAt: expect.any(Object),
            updatedAt: expect.any(Object),
          }),
        );
      });
    });

    describe('getCvResume', () => {
      it('should return NewCvResume for default CvResume initial value', () => {
        const formGroup = service.createCvResumeFormGroup(sampleWithNewData);

        const cvResume = service.getCvResume(formGroup);

        expect(cvResume).toMatchObject(sampleWithNewData);
      });

      it('should return NewCvResume for empty CvResume initial value', () => {
        const formGroup = service.createCvResumeFormGroup();

        const cvResume = service.getCvResume(formGroup);

        expect(cvResume).toMatchObject({});
      });

      it('should return ICvResume', () => {
        const formGroup = service.createCvResumeFormGroup(sampleWithRequiredData);

        const cvResume = service.getCvResume(formGroup);

        expect(cvResume).toMatchObject(sampleWithRequiredData);
      });
    });

    describe('resetForm', () => {
      it('passing ICvResume should not enable id FormControl', () => {
        const formGroup = service.createCvResumeFormGroup();
        expect(formGroup.controls.id.disabled).toBe(true);

        service.resetForm(formGroup, sampleWithRequiredData);

        expect(formGroup.controls.id.disabled).toBe(true);
      });

      it('passing NewCvResume should disable id FormControl', () => {
        const formGroup = service.createCvResumeFormGroup(sampleWithRequiredData);
        expect(formGroup.controls.id.disabled).toBe(true);

        service.resetForm(formGroup, { id: null });

        expect(formGroup.controls.id.disabled).toBe(true);
      });
    });
  });
});
