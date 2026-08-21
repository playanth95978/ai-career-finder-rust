import { beforeEach, describe, expect, it, vitest } from 'vitest';
import { HttpResponse } from '@angular/common/http';
import { provideHttpClientTesting } from '@angular/common/http/testing';
import { ComponentFixture, TestBed } from '@angular/core/testing';
import { ActivatedRoute } from '@angular/router';

import { provideTranslateService } from '@ngx-translate/core';
import { Subject, from, of } from 'rxjs';

import { ICvResume } from 'app/entities/cv-resume/cv-resume.model';
import { CvResumeService } from 'app/entities/cv-resume/service/cv-resume.service';
import { ICvResumeVersion } from '../cv-resume-version.model';
import { CvResumeVersionService } from '../service/cv-resume-version.service';

import { CvResumeVersionFormService } from './cv-resume-version-form.service';
import { CvResumeVersionUpdate } from './cv-resume-version-update';

describe('CvResumeVersion Management Update Component', () => {
  let comp: CvResumeVersionUpdate;
  let fixture: ComponentFixture<CvResumeVersionUpdate>;
  let activatedRoute: ActivatedRoute;
  let cvResumeVersionFormService: CvResumeVersionFormService;
  let cvResumeVersionService: CvResumeVersionService;
  let cvResumeService: CvResumeService;

  beforeEach(() => {
    TestBed.configureTestingModule({
      providers: [
        provideTranslateService(),
        provideHttpClientTesting(),
        {
          provide: ActivatedRoute,
          useValue: {
            params: from([{}]),
          },
        },
      ],
    });

    fixture = TestBed.createComponent(CvResumeVersionUpdate);
    activatedRoute = TestBed.inject(ActivatedRoute);
    cvResumeVersionFormService = TestBed.inject(CvResumeVersionFormService);
    cvResumeVersionService = TestBed.inject(CvResumeVersionService);
    cvResumeService = TestBed.inject(CvResumeService);

    comp = fixture.componentInstance;
  });

  describe('ngOnInit', () => {
    it('should call CvResume query and add missing value', () => {
      const cvResumeVersion: ICvResumeVersion = { id: 118 };
      const resume: ICvResume = { id: 8461 };
      cvResumeVersion.resume = resume;

      const cvResumeCollection: ICvResume[] = [{ id: 8461 }];
      vitest.spyOn(cvResumeService, 'query').mockReturnValue(of(new HttpResponse({ body: cvResumeCollection })));
      const additionalCvResumes = [resume];
      const expectedCollection: ICvResume[] = [...additionalCvResumes, ...cvResumeCollection];
      vitest.spyOn(cvResumeService, 'addCvResumeToCollectionIfMissing').mockReturnValue(expectedCollection);

      activatedRoute.data = of({ cvResumeVersion });
      comp.ngOnInit();

      expect(cvResumeService.query).toHaveBeenCalled();
      expect(cvResumeService.addCvResumeToCollectionIfMissing).toHaveBeenCalledWith(
        cvResumeCollection,
        ...additionalCvResumes.map(i => expect.objectContaining(i) as typeof i),
      );
      expect(comp.cvResumesSharedCollection()).toEqual(expectedCollection);
    });

    it('should update editForm', () => {
      const cvResumeVersion: ICvResumeVersion = { id: 118 };
      const resume: ICvResume = { id: 8461 };
      cvResumeVersion.resume = resume;

      activatedRoute.data = of({ cvResumeVersion });
      comp.ngOnInit();

      expect(comp.cvResumesSharedCollection()).toContainEqual(resume);
      expect(comp.cvResumeVersion).toEqual(cvResumeVersion);
    });
  });

  describe('save', () => {
    it('should call update service on save for existing entity', () => {
      // GIVEN
      const saveSubject = new Subject<ICvResumeVersion>();
      const cvResumeVersion = { id: 17476 };
      vitest.spyOn(cvResumeVersionFormService, 'getCvResumeVersion').mockReturnValue(cvResumeVersion);
      vitest.spyOn(cvResumeVersionService, 'update').mockReturnValue(saveSubject);
      vitest.spyOn(comp, 'previousState');
      activatedRoute.data = of({ cvResumeVersion });
      comp.ngOnInit();

      // WHEN
      comp.save();
      expect(comp.isSaving()).toEqual(true);
      saveSubject.next(cvResumeVersion);
      saveSubject.complete();

      // THEN
      expect(cvResumeVersionFormService.getCvResumeVersion).toHaveBeenCalled();
      expect(comp.previousState).toHaveBeenCalled();
      expect(cvResumeVersionService.update).toHaveBeenCalledWith(expect.objectContaining(cvResumeVersion));
      expect(comp.isSaving()).toEqual(false);
    });

    it('should call create service on save for new entity', () => {
      // GIVEN
      const saveSubject = new Subject<ICvResumeVersion>();
      const cvResumeVersion = { id: 17476 };
      vitest.spyOn(cvResumeVersionFormService, 'getCvResumeVersion').mockReturnValue({ id: null });
      vitest.spyOn(cvResumeVersionService, 'create').mockReturnValue(saveSubject);
      vitest.spyOn(comp, 'previousState');
      activatedRoute.data = of({ cvResumeVersion: null });
      comp.ngOnInit();

      // WHEN
      comp.save();
      expect(comp.isSaving()).toEqual(true);
      saveSubject.next(cvResumeVersion);
      saveSubject.complete();

      // THEN
      expect(cvResumeVersionFormService.getCvResumeVersion).toHaveBeenCalled();
      expect(cvResumeVersionService.create).toHaveBeenCalled();
      expect(comp.isSaving()).toEqual(false);
      expect(comp.previousState).toHaveBeenCalled();
    });

    it('should set isSaving to false on error', () => {
      // GIVEN
      const saveSubject = new Subject<ICvResumeVersion>();
      const cvResumeVersion = { id: 17476 };
      vitest.spyOn(cvResumeVersionService, 'update').mockReturnValue(saveSubject);
      vitest.spyOn(comp, 'previousState');
      activatedRoute.data = of({ cvResumeVersion });
      comp.ngOnInit();

      // WHEN
      comp.save();
      expect(comp.isSaving()).toEqual(true);
      saveSubject.error('This is an error!');

      // THEN
      expect(cvResumeVersionService.update).toHaveBeenCalled();
      expect(comp.isSaving()).toEqual(false);
      expect(comp.previousState).not.toHaveBeenCalled();
    });
  });

  describe('Compare relationships', () => {
    describe('compareCvResume', () => {
      it('should forward to cvResumeService', () => {
        const entity = { id: 8461 };
        const entity2 = { id: 15106 };
        vitest.spyOn(cvResumeService, 'compareCvResume');
        comp.compareCvResume(entity, entity2);
        expect(cvResumeService.compareCvResume).toHaveBeenCalledWith(entity, entity2);
      });
    });
  });
});
